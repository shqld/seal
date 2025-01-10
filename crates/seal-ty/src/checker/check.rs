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
							let init = var_declarator.init.as_ref().unwrap();

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
					_ => unimplemented!(),
				},
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
		let code = "let a = 1; a = 2;";
		let result = parse::parse(code).unwrap();

		let ast = result.program;
		let checker = Checker::new(ast);

		checker.check();
	}
}
