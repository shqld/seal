use std::{cell::Cell, ops::Deref};

use swc_ecma_ast::{BlockStmt, Function, ReturnStmt, Stmt};

use super::base::BaseChecker;
use super::scope::TyScope;

use crate::{Ty, TyKind, context::TyContext, symbol::Symbol};

#[derive(Debug)]
pub struct FunctionChecker<'tcx> {
	base: BaseChecker<'tcx>,
	ret: Ty<'tcx>,
	has_returned: Cell<bool>,
	root_scope: TyScope,
}

impl<'tcx> Deref for FunctionChecker<'tcx> {
	type Target = BaseChecker<'tcx>;

	fn deref(&self) -> &Self::Target {
		&self.base
	}
}

impl<'tcx> FunctionChecker<'tcx> {
	pub fn new(
		tcx: &'tcx TyContext<'tcx>,
		params: &[(Symbol, Ty<'tcx>)],
		ret: Ty<'tcx>,
	) -> FunctionChecker<'tcx> {
		let base = BaseChecker::new(tcx);

		for (name, ty) in params {
			base.add_var(name, *ty, false);
		}

		let root_scope = base.enter_new_scope();

		FunctionChecker {
			base,
			ret,
			has_returned: Cell::new(false),
			root_scope,
		}
	}

	pub fn into_result(self) -> Result<(), Vec<String>> {
		self.base.into_result()
	}

	pub fn check_function(&self, function: &Function) {
		let body = match &function.body {
			Some(body) => body,
			None => panic!("Function body is required"),
		};

		self.check_body(body);
	}

	pub fn check_body(&self, body: &BlockStmt) {
		for stmt in &body.stmts {
			self.check_stmt(stmt);
		}

		if !matches!(self.ret.kind(), TyKind::Void) && !self.has_returned.get() {
			self.add_error("function does not return".to_string());
		}

		assert_eq!(self.root_scope, self.get_current_scope());

		self.leave_current_scope();
	}

	pub fn check_stmt(&self, stmt: &Stmt) {
		match stmt {
			Stmt::Return(ReturnStmt { arg, .. }) => {
				let expected = self.ret;

				if let Some(arg) = arg {
					let actual = self.check_expr(arg);

					if !self.satisfies(expected, actual) {
						self.raise_type_error(expected, actual);
					}
				} else if !matches!(expected.kind(), TyKind::Void) {
					self.add_error("expected return value".to_string());
				}

				self.has_returned.set(true);
			}
			_ => self.base.check_stmt(stmt),
		}
	}
}
