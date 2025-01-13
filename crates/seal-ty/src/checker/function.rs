use std::{cell::Cell, ops::Deref};

use swc_ecma_ast::{BlockStmt, Function, ReturnStmt, Stmt};

use super::{base::BaseChecker, errors::Error};

use crate::checker::errors::ErrorKind;
use crate::{Ty, TyKind, context::TyContext, symbol::Symbol};

#[derive(Debug)]
pub struct FunctionChecker<'tcx> {
	base: BaseChecker<'tcx>,
	params: Vec<(Symbol, Ty<'tcx>)>,
	ret: Ty<'tcx>,
	has_returned: Cell<bool>,
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
		params: Vec<(Symbol, Ty<'tcx>)>,
		ret: Ty<'tcx>,
	) -> FunctionChecker<'tcx> {
		let base = BaseChecker::new(tcx);

		for (name, ty) in &params {
			base.add_var(name, *ty, false);
		}

		FunctionChecker {
			base,
			params,
			ret,
			has_returned: Cell::new(false),
		}
	}

	pub fn into_result(self) -> Result<(), Vec<Error<'tcx>>> {
		self.base.into_result()
	}

	pub fn check_function(&self, function: &Function) -> crate::kind::Function<'tcx> {
		let body = match &function.body {
			Some(body) => body,
			None => {
				self.add_error(ErrorKind::MissingBody);
				return crate::kind::Function::new(
					// TODO: check_function(self, ..)
					self.params.clone(),
					self.ret,
				);
			}
		};

		self.check_body(body)
	}

	pub fn check_body(&self, body: &BlockStmt) -> crate::kind::Function<'tcx> {
		for stmt in &body.stmts {
			self.check_stmt(stmt);
		}

		if !matches!(self.ret.kind(), TyKind::Void) && !self.has_returned.get() {
			self.add_error(ErrorKind::UnexpectedVoid);
		}

		crate::kind::Function::new(
			// TODO: check_function(self, ..)
			self.params.clone(),
			self.ret,
		)
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
					self.add_error(ErrorKind::UnexpectedVoid);
				}

				self.has_returned.set(true);
			}
			_ => self.base.check_stmt(stmt),
		}
	}
}
