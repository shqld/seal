use swc_ecma_ast::{
	Decl, Expr, ExprStmt, Lit, ModuleItem, Program, Stmt, TsKeywordTypeKind, TsSatisfiesExpr,
	TsType, VarDeclKind,
};

use super::Checker;
use crate::{Ty, TyKind};

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
			Expr::TsSatisfies(TsSatisfiesExpr { expr, type_ann, .. }) => {
				let expected_ty = self.build_tstype(type_ann);
				let actual_ty = self.build_expr(expr);

				if self.satisfies(expected_ty, actual_ty) {
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
					panic!("Type not found");
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
				_ => unimplemented!(),
			},
			_ => unimplemented!("{:#?}", tstype),
		})
	}

	fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
		expected != actual
	}
}

#[cfg(test)]
mod tests {
	use crate::checker::{context::TyContext, parse};

	use super::*;

	fn run(code: &'static str) {
		let result = parse::parse(code).unwrap();

		let ast = result.program;
		let tcx = TyContext::new();

		let checker = Checker::new(tcx);

		checker.check(&ast);
	}

	macro_rules! pass {
		($case_name:ident, $code:literal) => {
			#[test]
			fn $case_name() {
				run($code);
			}
		};
	}

	macro_rules! fail {
		($case_name:ident, $code:literal) => {
			#[should_panic]
			#[test]
			fn $case_name() {
				run($code);
			}
		};
	}

	pass!(
		let_,
		r#"
            let a = 1;
            a satisfies number;
        "#
	);

	pass!(
		let_uninitialized_,
		r#"
            let a; a = 1;
            a satisfies number;
        "#
	);

	fail!(
		assign_,
		r#"
            let a = 1;
            a = "hello";
        "#
	);
}
