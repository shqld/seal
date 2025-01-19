use std::collections::BTreeSet;

use swc_ecma_ast::{ExprStmt, IfStmt, Stmt};

use crate::TyKind;

use super::BaseChecker;

impl BaseChecker<'_> {
	pub fn check_stmt(&self, stmt: &Stmt) {
		match stmt {
			Stmt::Decl(decl) => self.check_decl(decl),
			Stmt::Expr(ExprStmt { expr, .. }) => {
				self.check_expr(expr);
			}
			Stmt::Return(_) => {
				unreachable!("Return statement must be handled in function context");
			}
			Stmt::If(IfStmt {
				test, cons, alt, ..
			}) => {
				let mut scopes = vec![self.new_scoped_checker()];
				let mut branches = vec![(test, cons)];

				let mut alt = alt.as_ref();

				while let Some(current_alt) = alt {
					scopes.push(self.new_scoped_checker());

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

				for (i, (test, cons)) in branches.iter().enumerate() {
					let checker = &scopes[i];
					let test_ty = self.check_expr(test);

					if let TyKind::Guard(name, narrowed_ty) = test_ty.kind() {
						let current_ty = self.get_var_ty(name).unwrap();

						checker.set_var(name, *narrowed_ty);

						if let Some(next_checker) = scopes.get(i + 1) {
							if let TyKind::Union(current) = current_ty.kind() {
								let narrowed_arms = match narrowed_ty.kind() {
									TyKind::Union(narrowed) => narrowed.arms(),
									_ => &BTreeSet::from([*narrowed_ty]),
								};

								let rest_arms =
									current.arms().difference(narrowed_arms).copied().collect();

								next_checker.set_var(name, self.tcx.new_union(rest_arms));
							}
						}
					}

					if let Stmt::Block(block) = cons.as_ref() {
						for stmt in &block.stmts {
							checker.check_stmt(stmt);
						}
					} else {
						checker.check_stmt(cons);
					}
				}
			}
			Stmt::Block(block) => {
				for stmt in &block.stmts {
					self.check_stmt(stmt);
				}
			}
			_ => todo!("{:#?}", stmt),
		}
	}
}
