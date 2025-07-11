use std::collections::BTreeSet;

use swc_ecma_ast::{
	BreakStmt, ContinueStmt, DoWhileStmt, ExprStmt, ForStmt, IfStmt, Stmt, SwitchStmt, ThrowStmt,
	TryStmt, WhileStmt,
};

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
					let test = self.check_expr(test);

					if let TyKind::Guard(name, narrowed_ty) = test.ty.kind() {
						let current = self.get_binding(name).unwrap();

						checker.set_ty(name, *narrowed_ty);

						if let Some(next_checker) = scopes.get(i + 1) {
							if let TyKind::Union(current) = current.ty.kind() {
								let narrowed_arms = match narrowed_ty.kind() {
									TyKind::Union(narrowed) => narrowed.arms(),
									_ => &BTreeSet::from([*narrowed_ty]),
								};

								let rest_arms =
									current.arms().difference(narrowed_arms).copied().collect();

								next_checker.set_ty(name, self.tcx.new_union(rest_arms));
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
			Stmt::While(WhileStmt { test, body, .. }) => {
				// Check test expression
				self.check_expr(test);

				// Check body in new scope
				let checker = self.new_scoped_checker();
				checker.check_stmt(body);
			}
			Stmt::DoWhile(DoWhileStmt { test, body, .. }) => {
				// Check body first (since it always executes at least once)
				let checker = self.new_scoped_checker();
				checker.check_stmt(body);

				// Then check test expression
				self.check_expr(test);
			}
			Stmt::For(ForStmt {
				init,
				test,
				update,
				body,
				..
			}) => {
				// Create new scope for the entire for loop
				let checker = self.new_scoped_checker();

				// Check init (variable declaration or expression)
				if let Some(init) = init {
					match init {
						swc_ecma_ast::VarDeclOrExpr::VarDecl(decl) => {
							checker.check_decl(&swc_ecma_ast::Decl::Var(decl.clone()))
						}
						swc_ecma_ast::VarDeclOrExpr::Expr(expr) => {
							checker.check_expr(expr);
						}
					}
				}

				// Check test expression
				if let Some(test) = test {
					checker.check_expr(test);
				}

				// Check update expression
				if let Some(update) = update {
					checker.check_expr(update);
				}

				// Check body
				checker.check_stmt(body);
			}
			Stmt::Break(BreakStmt { .. }) => {
				// Break statement - no type checking needed
			}
			Stmt::Continue(ContinueStmt { .. }) => {
				// Continue statement - no type checking needed
			}
			Stmt::Switch(SwitchStmt {
				discriminant,
				cases,
				..
			}) => {
				// Check discriminant expression
				let _discriminant_value = self.check_expr(discriminant);

				// Check each case
				for case in cases {
					let checker = self.new_scoped_checker();

					// Check case test expression if it exists
					if let Some(test) = &case.test {
						checker.check_expr(test);
					}

					// Check case body
					for stmt in &case.cons {
						checker.check_stmt(stmt);
					}
				}
			}
			Stmt::Throw(ThrowStmt { arg, .. }) => {
				// Check the thrown expression
				self.check_expr(arg);
			}
			Stmt::Try(try_stmt) => {
				let TryStmt {
					block,
					handler,
					finalizer,
					..
				} = try_stmt.as_ref();
				// Check try block
				for stmt in &block.stmts {
					self.check_stmt(stmt);
				}

				// Check catch handler if it exists
				if let Some(handler) = handler {
					let checker = self.new_scoped_checker();

					// Check catch parameter if it exists
					if let Some(swc_ecma_ast::Pat::Ident(ident)) = &handler.param {
						// Check if there's a type annotation - this should be an error
						if ident.type_ann.is_some() {
							self.add_error_with_span(crate::checker::errors::ErrorKind::CatchParameterCannotHaveTypeAnnotation, ident.span);
						}

						// Always bind the catch parameter as unknown type
						let name = crate::symbol::Symbol::new(ident.to_id());
						let catch_value =
							checker.add_local(self.constants.unknown, crate::sir::Value::Param);
						checker.set_binding(
							&name,
							Some(catch_value),
							self.constants.unknown,
							false,
						);
					}

					// Check catch body
					for stmt in &handler.body.stmts {
						checker.check_stmt(stmt);
					}
				}

				// Check finally block if it exists
				if let Some(finalizer) = finalizer {
					for stmt in &finalizer.stmts {
						self.check_stmt(stmt);
					}
				}
			}
			_ => todo!("{:#?}", stmt),
		}
	}
}
