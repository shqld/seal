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
				unreachable!("Return statement should be handled in function context");
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
}
