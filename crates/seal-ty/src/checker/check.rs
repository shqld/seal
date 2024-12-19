use super::Checker;
use crate::{Ty, TyKind};
use swc_ecma_ast::{Decl, Expr, ExprStmt, Lit, ModuleItem, Program, Stmt, VarDeclKind};

impl<'tcx> Checker<'tcx> {
	pub fn check(&'tcx self) {
		let stmts = match &self.tcx.ast {
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
			_ => unimplemented!("{:#?}", decl),
		}
	}

	fn build_expr(&'tcx self, expr: &Expr) -> Ty<'tcx> {
		match expr {
			Expr::Assign(assign) => {
				let id = assign.left.clone().expect_simple().expect_ident().to_id();

				let actual_ty = self.build_expr(&assign.right);

				let ty = if let Some(expected_ty) = self.tcx.get_ty(&id) {
					if self.satisfies(expected_ty, actual_ty) {
						panic!("Type mismatch");
					}

					expected_ty
				} else {
					self.tcx.set_ty(id, actual_ty);
					actual_ty
				};

				ty
			}
			Expr::Lit(lit) => self.tcx.new_ty(match lit {
				Lit::Bool(_) => TyKind::Boolean,
				Lit::Num(_) => TyKind::Number,
				Lit::Str(_) => TyKind::String,
				_ => unimplemented!(),
			}),
			_ => unimplemented!("{:#?}", expr),
		}
	}

	fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
		expected != actual
	}
}

#[cfg(test)]
mod tests {
	use crate::checker::parse;

	use super::*;

	#[test]
	fn test_checker() {
		let code = "let a; a = 1;";
		let result = parse::parse(code).unwrap();

		let ast = result.program;
		let checker = Checker::new(ast);

		checker.check();
	}
}
