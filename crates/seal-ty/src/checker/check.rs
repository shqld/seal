use swc_ecma_ast::{Decl, Expr, Lit, ModuleItem, Program, Stmt, VarDeclKind};

use super::Checker;
use crate::TyKind;

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
			match stmt {
				Stmt::Decl(decl) => match decl {
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
					_ => unimplemented!(),
				},
				Stmt::Expr(stmt) => {
					// dbg!(&stmt.expr);
					match &*stmt.expr {
						Expr::Assign(assign) => {
							let id = assign.left.clone().expect_simple().expect_ident().to_id();
							let kind = match &*assign.right {
								Expr::Lit(lit) => match lit {
									Lit::Bool(_) => TyKind::Boolean,
									Lit::Num(_) => TyKind::Number,
									Lit::Str(_) => TyKind::String,
									_ => unimplemented!(),
								},
								_ => unimplemented!(),
							};

							let ty = self.tcx.new_ty(kind);

							if let Some(expected_ty) = self.tcx.get_ty(&id) {
								if expected_ty != ty {
									panic!("Type mismatch");
								}
							}

							self.tcx.set_ty(id, ty);
						}
						_ => unimplemented!("{:#?}", stmt.expr),
					}
				}
				_ => unimplemented!("{:#?}", stmt),
			}
		}
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
