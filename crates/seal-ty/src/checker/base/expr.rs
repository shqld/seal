use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
	AssignExpr, AssignTarget, BinExpr, BinaryOp, BlockStmtOrExpr, Decl, Expr, Lit, MemberExpr,
	MemberProp, Pat, Prop, PropOrSpread, SimpleAssignTarget, TsSatisfiesExpr, UnaryOp, VarDeclKind,
};

use crate::{Ty, TyKind, checker::function::FunctionChecker, symbol::Symbol};

use super::BaseChecker;

impl<'tcx> BaseChecker<'tcx> {
	pub fn check_decl(&self, decl: &Decl) {
		match decl {
			Decl::Var(var) => {
				let is_const = match var.kind {
					VarDeclKind::Var => panic!("Var is not supported"),
					VarDeclKind::Const => true,
					VarDeclKind::Let => false,
				};

				for var_declarator in &var.decls {
					let binding = match &var_declarator.name {
						Pat::Ident(ident) => ident,
						_ => unimplemented!("{:#?}", var_declarator.name),
					};
					let ty = binding
						.type_ann
						.as_ref()
						.map(|type_ann| self.build_ts_type(&type_ann.type_ann))
						.unwrap_or_else(|| self.constants.lazy);

					let binding = Symbol::new(binding.to_id());

					if let Some(init) = &var_declarator.init {
						let expected = ty;
						let actual = self.check_expr(init);

						// if no type is specified to the declaration, replace with actual type
						if let TyKind::Lazy = ty.kind() {
							self.add_var(&binding, actual, !is_const);
							return;
						}

						if !self.satisfies(expected, actual) {
							self.raise_type_error(expected, actual);
						}
					} else if is_const {
						panic!("Const variable must be initialized");
					};

					self.add_var(&binding, ty, !is_const);
				}
			}
			Decl::Fn(_) => {
				unreachable!(
					"Function declaration should be handled in module or function context"
				);
			}

			_ => unimplemented!("{:#?}", decl),
		}
	}

	pub fn check_expr(&self, expr: &Expr) -> Ty<'tcx> {
		match expr {
			Expr::Assign(AssignExpr { left, right, .. }) => {
				let binding = match &left {
					AssignTarget::Simple(target) => match &target {
						SimpleAssignTarget::Ident(ident) => ident,
						_ => unimplemented!("{:#?}", target),
					},
					_ => unimplemented!("{:#?}", left),
				};
				let binding = Symbol::new(binding.to_id());

				if !self.get_var_is_assignable(&binding).unwrap() {
					panic!("Cannot assign to immutable variable");
				}

				let expected = self.get_var_ty(&binding).unwrap();
				let actual = self.check_expr(right);

				// if no type is specified to the declaration, replace with actual type
				if let TyKind::Lazy = expected.kind() {
					self.add_var(&binding, actual, true);
					return actual;
				}

				if !self.satisfies(expected, actual) {
					self.raise_type_error(expected, actual);
				}

				// TODO: actual?
				expected
			}
			Expr::TsSatisfies(TsSatisfiesExpr { expr, type_ann, .. }) => {
				let expected = self.build_ts_type(type_ann);
				let actual = self.check_expr(expr);

				if !self.satisfies(expected, actual) {
					self.raise_type_error(expected, actual);
				}

				actual
			}
			Expr::Lit(lit) => match lit {
				Lit::Bool(_) => self.constants.boolean,
				Lit::Num(_) => self.constants.number,
				Lit::Str(value) => self.tcx.new_const_string(value.value.clone()),
				_ => unimplemented!("{:#?}", lit),
			},
			Expr::Ident(ident) => {
				let name = Symbol::new(ident.to_id());

				if let Some(ty) = self.get_var_ty(&name) {
					ty
				} else {
					panic!("Type not found: {:?}", name);
				}
			}

			Expr::Unary(unary) => {
				self.check_expr(&unary.arg);

				match unary.op {
					UnaryOp::TypeOf => self.constants.type_of,
					_ => unimplemented!("{:#?}", unary),
				}
			}
			Expr::Bin(BinExpr {
				op, left, right, ..
			}) => {
				let left_ty = self.check_expr(left);
				let right_ty = self.check_expr(right);

				match op {
					BinaryOp::EqEqEq => {
						if !self.overlaps(left_ty, right_ty) {
							// TS(2367)
							self.add_error(format!("This comparison appears to be unintentional because the types '{left_ty}' and '{right_ty}' have no overlap"));

							return self.constants.err;
						}

						if let Some(narrowed_ty) = self.narrow(left, right) {
							return narrowed_ty;
						}

						self.constants.boolean
					}
					_ => unimplemented!("{:#?}", op),
				}
			}
			Expr::Member(MemberExpr { obj, prop, .. }) => {
				let prop = match &prop {
					MemberProp::Ident(ident) => ident.sym.clone(),
					_ => unimplemented!("{:#?}", prop),
				};

				let obj_ty = self.check_expr(obj);

				match obj_ty.kind() {
					TyKind::Object(obj) => match obj.fields().get(&prop) {
						Some(ty) => *ty,
						None => {
							// TS(2339)
							self.add_error(format!(
								"Property '{prop}' does not exist on type '{obj_ty}'",
							));
							self.constants.err
						}
					},
					TyKind::Union(uni) => {
						let mut prop_arms = BTreeSet::new();

						for arm in uni.arms() {
							if let TyKind::Object(obj) = arm.kind() {
								if let Some(prop) = obj.get_prop(&prop) {
									prop_arms.insert(prop);
									continue;
								}
							}

							// TS(2339)
							self.add_error(format!(
								"Property '{prop}' does not exist on type '{arm}'",
							));

							return self.constants.err;
						}

						self.tcx.new_union(prop_arms)
					}
					_ => {
						// TS(2339)
						self.add_error(format!(
							"Property '{prop}' does not exist on type '{obj_ty}'",
						));
						self.constants.err
					}
				}
			}
			Expr::Object(obj) => {
				let mut fields = BTreeMap::new();

				for prop in &obj.props {
					match prop {
						PropOrSpread::Prop(prop) => match prop.as_ref() {
							Prop::KeyValue(kv) => {
								let key = kv.key.as_ident().unwrap().sym.clone();
								let ty = self.check_expr(&kv.value);

								fields.insert(key, ty);
							}
							_ => unimplemented!("{:#?}", prop),
						},
						_ => unimplemented!("{:#?}", prop),
					}
				}

				self.tcx.new_object(fields)
			}
			Expr::Arrow(closure) => {
				let mut params = vec![];

				for param in &closure.params {
					match param {
						Pat::Ident(ident) => {
							let name = Symbol::new(ident.to_id());
							let ty = self.get_var_ty(&name).unwrap();

							params.push((name, ty));
						}
						_ => unimplemented!("{:#?}", param),
					};
				}

				match closure.body.as_ref() {
					BlockStmtOrExpr::Expr(body) => {
						let ret = self.check_expr(body);

						if let Some(return_type) = &closure.return_type {
							let expected = self.build_ts_type(&return_type.type_ann);

							if !self.satisfies(expected, ret) {
								self.raise_type_error(expected, ret);
							}
						}

						self.tcx.new_function(params, ret)
					}
					BlockStmtOrExpr::BlockStmt(body) => {
						let ret = match &closure.return_type {
							Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
							None => self.constants.void,
						};

						let checker = FunctionChecker::new(self.tcx, &params, ret);

						for (name, var) in self.vars.borrow().iter() {
							checker.add_var(name, var.ty, var.is_assignable);
						}

						checker.check_body(body);

						if let Err(errors) = checker.into_result() {
							for error in errors {
								self.add_error(error);
							}
						}

						self.tcx.new_function(params, ret)
					}
				}
			}
			_ => unimplemented!("{:#?}", expr),
		}
	}
}
