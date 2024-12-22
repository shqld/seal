use swc_ecma_ast::{
	Decl, Expr, ExprStmt, FnDecl, Function, Lit, ModuleItem, Pat, Program, ReturnStmt, Stmt,
	TsFnOrConstructorType, TsKeywordTypeKind, TsSatisfiesExpr, TsType, VarDeclKind,
};

use super::Checker;
use crate::{Ty, TyKind, kind::FunctionTy};

impl<'tcx> Checker<'tcx> {
	pub fn check(&'tcx self, ast: &Program) {
		let stmts = match &ast {
			Program::Script(script) => &script.body,
			Program::Module(module) => &module
				.body
				.iter()
				.filter_map(|item| match item {
					ModuleItem::Stmt(stmt) => Some(stmt.clone()),
					_ => None,
				})
				.collect::<Vec<_>>(),
		};

		for stmt in stmts {
			self.check_stmt(stmt);
		}
	}

	fn check_stmt(&'tcx self, stmt: &Stmt) {
		match stmt {
			Stmt::Decl(decl) => self.check_decl(decl),
			Stmt::Expr(ExprStmt { expr, .. }) => {
				self.build_expr(expr);
			}
			Stmt::Return(ReturnStmt { arg, .. }) => {
				let ret_ty = match arg {
					Some(arg) => self.build_expr(arg),
					None => self.tcx.new_ty(TyKind::Void),
				};

				let expected_ty = self.get_current_function_return_ty().unwrap();

				if !self.satisfies(expected_ty, ret_ty) {
					panic!("Return type mismatch");
				}

				self.set_function_has_returned();
			}
			_ => unimplemented!("{:#?}", stmt),
		}
	}

	fn check_decl(&'tcx self, decl: &Decl) {
		match decl {
			Decl::Var(var) => {
				let _is_const = match var.kind {
					VarDeclKind::Var => panic!("Var is not supported"),
					VarDeclKind::Const => true,
					VarDeclKind::Let => false,
				};

				for var_declarator in &var.decls {
					if let Some(init) = &var_declarator.init {
						let kind = match &**init {
							Expr::Lit(lit) => match lit {
								Lit::Bool(_) => TyKind::Boolean,
								Lit::Num(_) => TyKind::Number,
								Lit::Str(_) => TyKind::String,
								_ => unimplemented!(),
							},
							_ => unimplemented!(),
						};

						let ty = self.tcx.new_ty(kind);
						let id = var_declarator.name.clone().expect_ident().to_id();

						self.tcx.set_ty(id, ty);
					}
				}
			}
			Decl::Fn(FnDecl {
				ident, function, ..
			}) => {
				let id = ident.to_id();

				let ty = self.build_function(function);

				self.tcx.set_ty(id, ty);
			}
			_ => unimplemented!("{:#?}", decl),
		}
	}

	fn build_function(&'tcx self, function: &Function) -> Ty<'tcx> {
		let return_ty = function
			.return_type
			.as_ref()
			.map(|rt| self.build_tstype(&rt.type_ann))
			.unwrap_or(self.tcx.new_ty(TyKind::Void));

		self.push_function_scope(return_ty);

		let mut param_tys = vec![];
		for param in &function.params {
			match &param.pat {
				Pat::Ident(ident) => {
					let id = ident.to_id();
					let ty = match &ident.type_ann {
						Some(ty) => self.build_tstype(&ty.type_ann),
						None => self.tcx.new_infer_ty(),
					};

					self.tcx.set_ty(id, ty);
					param_tys.push(ty);
				}
				_ => unimplemented!("{:#?}", param),
			}
		}

		let body = match &function.body {
			Some(body) => body,
			None => panic!("Function body is required"),
		};

		for stmt in &body.stmts {
			self.check_stmt(stmt);
		}

		if !self.get_current_function_has_returned() && return_ty != self.tcx.new_ty(TyKind::Void) {
			panic!("Function does not return");
		}

		let ty = self.tcx.new_ty(TyKind::Function(FunctionTy {
			params: param_tys,
			ret: return_ty,
		}));

		self.pop_function_scope();

		ty
	}

	fn build_expr(&'tcx self, expr: &Expr) -> Ty<'tcx> {
		match expr {
			Expr::Assign(assign) => {
				let id = assign.left.clone().expect_simple().expect_ident().to_id();

				let actual_ty = self.build_expr(&assign.right);

				let ty = if let Some(expected_ty) = self.tcx.get_ty(&id) {
					if !self.satisfies(expected_ty, actual_ty) {
						panic!("Type mismatch");
					}

					expected_ty
				} else {
					self.tcx.set_ty(id, actual_ty);
					actual_ty
				};

				ty
			}
			Expr::TsSatisfies(TsSatisfiesExpr { expr, type_ann, .. }) => {
				let expected_ty = self.build_tstype(type_ann);
				let actual_ty = self.build_expr(expr);

				if !self.satisfies(expected_ty, actual_ty) {
					dbg!(expected_ty, actual_ty);
					panic!("Type mismatch");
				}

				actual_ty
			}
			Expr::Lit(lit) => self.tcx.new_ty(match lit {
				Lit::Bool(_) => TyKind::Boolean,
				Lit::Num(_) => TyKind::Number,
				Lit::Str(_) => TyKind::String,
				_ => unimplemented!(),
			}),
			Expr::Ident(ident) => {
				let id = ident.to_id();

				if let Some(ty) = self.tcx.get_ty(&id) {
					ty
				} else {
					panic!("Type not found: {:?}", ident.sym);
				}
			}
			_ => unimplemented!("{:#?}", expr),
		}
	}

	fn build_tstype(&'tcx self, tstype: &TsType) -> Ty<'tcx> {
		self.tcx.new_ty(match tstype {
			TsType::TsKeywordType(keyword) => match keyword.kind {
				TsKeywordTypeKind::TsNumberKeyword => TyKind::Number,
				TsKeywordTypeKind::TsStringKeyword => TyKind::String,
				TsKeywordTypeKind::TsBooleanKeyword => TyKind::Boolean,
				TsKeywordTypeKind::TsVoidKeyword => TyKind::Void,
				_ => unimplemented!(),
			},
			TsType::TsFnOrConstructorType(fn_or_constructor) => match fn_or_constructor {
				TsFnOrConstructorType::TsFnType(fn_) => {
					let return_ty = self.build_tstype(&fn_.type_ann.type_ann);

					let mut param_tys = vec![];
					for param in &fn_.params {
						let ty = self.build_tstype(
							// TODO:
							&param.clone().expect_ident().type_ann.unwrap().type_ann,
						);
						param_tys.push(ty);
					}

					TyKind::Function(FunctionTy {
						params: param_tys,
						ret: return_ty,
					})
				}
				_ => unimplemented!(),
			},
			_ => unimplemented!("{:#?}", tstype),
		})
	}

	fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
		use TyKind::*;

		match (expected.kind(), actual.kind()) {
			(_, Infer(id)) => match self.tcx.infer.resolve_ty(*id) {
				Some(actual) => self.satisfies(expected, actual),
				None => {
					self.tcx.infer.add_constraint(*id, expected);
					// TODO: unify when function scope ends
					self.tcx.infer.unify(*id, expected);

					true
				}
			},
			(Function(expected), Function(actual)) => {
				for (expected, actual) in expected.params.iter().zip(&actual.params) {
					if !self.satisfies(*expected, *actual) {
						return false;
					}
				}

				self.satisfies(expected.ret, actual.ret)
			}
			_ => expected == actual,
		}
	}
}
