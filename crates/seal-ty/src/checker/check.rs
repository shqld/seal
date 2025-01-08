use std::collections::{BTreeMap, BTreeSet};

use swc_ecma_ast::{
	AssignExpr, AssignTarget, BinExpr, BinaryOp, Decl, Expr, ExprStmt, FnDecl, IfStmt, Lit,
	MemberExpr, MemberProp, ModuleItem, Pat, Program, Prop, PropOrSpread, ReturnStmt,
	SimpleAssignTarget, Stmt, TsFnOrConstructorType, TsFnParam, TsKeywordTypeKind, TsLit,
	TsLitType, TsSatisfiesExpr, TsType, TsTypeLit, TsUnionOrIntersectionType, UnaryOp, VarDeclKind,
};

use crate::{Ty, TyKind, symbol::Symbol};

use super::Checker;

impl<'tcx> Checker<'tcx> {
	pub fn check(self, ast: &Program) -> Result<(), Vec<String>> {
		self.start_function(&Symbol::new_main(), vec![], self.constants.void);

		match &ast {
			Program::Script(script) => {
				for stmt in &script.body {
					match stmt {
						Stmt::Return(_) => {
							// TS(1108)
							panic!("Return statement is not allowed in the main function");
						}
						_ => self.check_stmt(stmt),
					}
				}
			}
			Program::Module(module) => {
				for module_item in &module.body {
					match module_item {
						ModuleItem::Stmt(stmt) => match stmt {
							Stmt::Return(_) => {
								// TS(1108)
								panic!("Return statement is not allowed in the main function");
							}
							_ => self.check_stmt(stmt),
						},
						_ => unimplemented!("{:#?}", module_item),
					}
				}
			}
		};

		self.finish_function();

		if self.errors.borrow().is_empty() {
			Ok(())
		} else {
			Err(self.errors.into_inner())
		}
	}

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
						.map(|type_ann| self.build_tstype(&type_ann.type_ann))
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

			Decl::Fn(FnDecl {
				ident, function, ..
			}) => {
				let name = Symbol::new(ident.to_id());
				let mut params = vec![];

				for param in &function.params {
					match &param.pat {
						Pat::Ident(ident) => {
							let name = Symbol::new(ident.to_id());

							let ty = match &ident.type_ann {
								Some(type_ann) => self.build_tstype(&type_ann.type_ann),
								None => {
									panic!("Param type annotation is required");
								}
							};

							params.push((name, ty));
						}
						_ => unimplemented!("{:#?}", param),
					}
				}

				let ret_ty = match &function.return_type {
					Some(type_ann) => self.build_tstype(&type_ann.type_ann),
					None => {
						// NOTE: seal does't infer the return type
						self.constants.void
					}
				};

				self.start_function(&name, params, ret_ty);

				let body = match &function.body {
					Some(body) => body,
					None => panic!("Function body is required"),
				};

				for stmt in &body.stmts {
					self.check_stmt(stmt);
				}

				if !matches!(ret_ty.kind(), TyKind::Void)
					&& !self.get_current_function_has_returned()
				{
					self.add_error("function does not return".to_string());
				}

				self.finish_function();
			}
			_ => unimplemented!("{:#?}", decl),
		}
	}

	pub fn check_stmt(&self, stmt: &Stmt) {
		match stmt {
			Stmt::Decl(decl) => self.check_decl(decl),
			Stmt::Expr(ExprStmt { expr, .. }) => {
				self.check_expr(expr);
			}
			Stmt::Return(ReturnStmt { arg, .. }) => {
				let expected = self.get_current_function_ret();

				if let Some(arg) = arg {
					let actual = self.check_expr(arg);

					if !self.satisfies(expected, actual) {
						self.raise_type_error(expected, actual);
					}
				} else if !matches!(expected.kind(), TyKind::Void) {
					self.add_error("expected return value".to_string());
				}

				self.set_current_function_has_returned(true);
			}
			Stmt::If(IfStmt {
				test, cons, alt, ..
			}) => {
				let mut branches = vec![(test, cons)];
				let mut alt = alt.as_ref();

				while let Some(current_alt) = alt {
					if let Stmt::If(IfStmt {
						test,
						cons,
						alt: next_alt,
						..
					}) = current_alt.as_ref()
					{
						branches.push((test, cons));
						alt = next_alt.as_ref();
					} else {
						break;
					}
				}

				let mut next_scope = self.get_current_scope().next();

				for (test, cons) in branches {
					let test_ty = self.check_expr(test);

					if let TyKind::Guard(name, narrowed_ty) = test_ty.kind() {
						self.add_scoped_ty(name, next_scope, *narrowed_ty);

						let current_ty = self.get_var_ty(name).unwrap();

						if let TyKind::Union(current) = current_ty.kind() {
							let narrowed_arms = match narrowed_ty.kind() {
								TyKind::Union(narrowed) => narrowed.arms(),
								_ => &BTreeSet::from([*narrowed_ty]),
							};

							let rest_arms =
								current.arms().difference(narrowed_arms).copied().collect();

							self.add_scoped_ty(
								name,
								next_scope.next(),
								self.tcx.new_union(rest_arms),
							);
						}
					}

					self.enter_new_scope();
					if let Stmt::Block(block) = cons.as_ref() {
						for stmt in &block.stmts {
							self.check_stmt(stmt);
						}
					} else {
						self.check_stmt(cons);
					}
					self.leave_current_scope();

					next_scope = next_scope.next();
				}
			}
			Stmt::Block(block) => {
				for stmt in &block.stmts {
					self.check_stmt(stmt);
				}
			}
			_ => unimplemented!("{:#?}", stmt),
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
				let expected = self.build_tstype(type_ann);
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
			_ => unimplemented!("{:#?}", expr),
		}
	}

	pub fn raise_type_error(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) {
		use TyKind::*;

		if matches!(actual.kind(), String(Some(_))) && !matches!(expected.kind(), String(_)) {
			self.add_error(format!(
				"expected '{expected}', got '{}'",
				TyKind::String(None)
			));
		} else {
			self.add_error(format!("expected '{expected}', got '{actual}'"));
		}
	}

	pub fn build_tstype(&self, tstype: &TsType) -> Ty<'tcx> {
		match tstype {
			TsType::TsKeywordType(keyword) => match keyword.kind {
				TsKeywordTypeKind::TsNumberKeyword => self.constants.number,
				TsKeywordTypeKind::TsStringKeyword => self.constants.string,
				TsKeywordTypeKind::TsBooleanKeyword => self.constants.boolean,
				TsKeywordTypeKind::TsVoidKeyword => self.constants.void,
				TsKeywordTypeKind::TsNeverKeyword => self.constants.never,
				_ => unimplemented!(),
			},
			TsType::TsFnOrConstructorType(fn_or_constructor) => match fn_or_constructor {
				TsFnOrConstructorType::TsFnType(fn_) => {
					let ret_ty = self.build_tstype(&fn_.type_ann.type_ann);

					let mut param_tys = vec![];
					for param in &fn_.params {
						let ty = match param {
							TsFnParam::Ident(ident) => {
								self.build_tstype(&ident.type_ann.as_ref().unwrap().type_ann)
							}
							_ => unimplemented!("{:#?}", param),
						};
						param_tys.push(ty);
					}

					self.tcx.new_function(param_tys, ret_ty)
				}
				_ => unimplemented!(),
			},
			TsType::TsUnionOrIntersectionType(ty) => match ty {
				TsUnionOrIntersectionType::TsUnionType(ty) => self.tcx.new_union(
					ty.types
						.iter()
						.map(|ty| self.build_tstype(ty))
						.collect::<BTreeSet<_>>(),
				),
				TsUnionOrIntersectionType::TsIntersectionType(_) => unimplemented!(),
			},
			TsType::TsLitType(TsLitType { lit, .. }) => match lit {
				TsLit::Str(str) => self.tcx.new_const_string(str.value.clone()),
				_ => unimplemented!("{:#?}", lit),
			},
			TsType::TsTypeLit(TsTypeLit { members, .. }) => {
				let mut fields = BTreeMap::new();
				for member in members {
					match member {
						swc_ecma_ast::TsTypeElement::TsPropertySignature(prop) => {
							let name = prop.key.as_ident().unwrap().sym.clone();
							let ty = self.build_tstype(&prop.type_ann.as_ref().unwrap().type_ann);
							fields.insert(name, ty);
						}
						_ => unimplemented!("{:#?}", member),
					}
				}

				self.tcx.new_object(fields)
			}
			_ => unimplemented!("{:#?}", tstype),
		}
	}
}
