use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
	AssignExpr, AssignTarget, BinExpr, BinaryOp, BlockStmtOrExpr, Expr, ExprOrSpread, Lit,
	MemberExpr, MemberProp, NewExpr, Pat, Prop, PropOrSpread, SimpleAssignTarget, TsSatisfiesExpr,
	UnaryOp,
};

use crate::{Ty, TyKind, checker::function::FunctionChecker, symbol::Symbol};

use super::BaseChecker;

impl<'tcx> BaseChecker<'tcx> {
	pub fn check_expr(&self, expr: &Expr) -> Ty<'tcx> {
		match expr {
			Expr::Assign(AssignExpr { left, right, .. }) => {
				let binding = match &left {
					AssignTarget::Simple(target) => match &target {
						SimpleAssignTarget::Ident(ident) => ident,
						_ => todo!("{:#?}", target),
					},
					_ => todo!("{:#?}", left),
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
				_ => todo!("{:#?}", lit),
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
					_ => todo!("{:#?}", unary),
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
					_ => todo!("{:#?}", op),
				}
			}
			Expr::Member(MemberExpr { obj, prop, .. }) => {
				let key = match &prop {
					MemberProp::Ident(ident) => ident.sym.clone(),
					_ => todo!("{:#?}", prop),
				};

				let obj_ty = self.check_expr(obj);

				match obj_ty.kind() {
					TyKind::Object(obj) => match obj.fields().get(&key) {
						Some(ty) => *ty,
						None => {
							// TS(2339)
							self.add_error(format!(
								"Property '{key}' does not exist on type '{obj_ty}'",
							));
							self.constants.err
						}
					},
					TyKind::Interface(interface) => match interface.fields().get(&key) {
						Some(ty) => *ty,
						None => {
							// TS(2339)
							self.add_error(format!(
								"Property '{key}' does not exist on type '{obj_ty}'",
							));
							self.constants.err
						}
					},
					TyKind::Union(uni) => {
						let mut prop_arms = BTreeSet::new();

						for arm in uni.arms() {
							if let TyKind::Object(obj) = arm.kind() {
								if let Some(prop) = obj.get_prop(&key) {
									prop_arms.insert(prop);
									continue;
								}
							}

							// TS(2339)
							self.add_error(format!(
								"Property '{key}' does not exist on type '{arm}'",
							));

							return self.constants.err;
						}

						self.tcx.new_union(prop_arms)
					}
					_ => {
						// TODO: other message? (e.g. "Property access on non-object is not allowed.")
						// TS(2339)
						self.add_error(format!(
							"Property '{key}' does not exist on type '{obj_ty}'",
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
							_ => todo!("{:#?}", prop),
						},
						_ => todo!("{:#?}", prop),
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
						_ => todo!("{:#?}", param),
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

						self.tcx.new_function(crate::kind::Function { params, ret })
					}
					BlockStmtOrExpr::BlockStmt(body) => {
						let ret = match &closure.return_type {
							Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
							None => self.constants.void,
						};

						let checker = FunctionChecker::new(self.tcx, params, ret);

						for (name, var) in self.vars.borrow().iter() {
							checker.add_var(name, var.ty, var.is_assignable);
						}

						let function = checker.check_body(body);

						if let Err(errors) = checker.into_result() {
							for error in errors {
								self.add_error(error);
							}
						}

						self.tcx.new_function(function)
					}
				}
			}
			Expr::New(NewExpr { callee, args, .. }) => {
				let callee = self.check_expr(callee);

				let class = match callee.kind() {
					TyKind::Class(class) => class,
					_ => {
						// TS(2351)
						self.add_error("This expression is not constructable.".to_owned());
						return self.constants.err;
					}
				};

				let args = match args {
					Some(args) => args,
					None => {
						self.add_error("Arguments must follow after 'new'.".to_owned());
						return self.constants.err;
					}
				};

				if let Some(ctor) = class.ctor() {
					let params = &ctor.params;

					if params.len() != args.len() {
						// TS(2554)
						self.add_error(format!(
							"Expected {} arguments, but got {}",
							params.len(),
							args.len(),
						));
						return self.constants.err;
					}

					let args = args.iter().map(|ExprOrSpread { expr, spread }| {
						if spread.is_some() {
							todo!()
						}

						self.check_expr(expr)
					});

					for ((_, param), arg) in params.iter().zip(args) {
						if !self.satisfies(*param, arg) {
							self.raise_type_error(*param, arg);
						}
					}
				} else if !args.is_empty() {
					// TS(2554)
					self.add_error(format!("Expected 0 arguments, but got {}", args.len(),));
					return self.constants.err;
				}

				self.tcx.new_interface(class.interface().clone())
			}
			_ => todo!("{:#?}", expr),
		}
	}
}
