use std::collections::HashMap;
use std::{cell::Cell, ops::Deref};

use swc_ecma_ast::{BlockStmt, Function, ReturnStmt, Stmt};

use super::{base::BaseChecker, errors::Error};

use crate::checker::errors::ErrorKind;
use crate::sir::{self, Value};
use crate::{Ty, TyKind, context::TyContext, symbol::Symbol};

pub struct FunctionCheckerResult<'tcx> {
	pub ty: crate::kind::Function<'tcx>,
	pub def: sir::Func,
	pub errors: Vec<Error<'tcx>>,
}

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
			let param = base.add_local(*ty, Value::Param);
			base.set_binding(name, Some(param), *ty, false);
		}

		FunctionChecker {
			base,
			params,
			ret,
			has_returned: Cell::new(false),
		}
	}

	pub fn check_function(self, function: &Function) -> FunctionCheckerResult<'tcx> {
		let body = match &function.body {
			Some(body) => body,
			None => {
				return FunctionCheckerResult {
					ty: crate::kind::Function::new(
						// TODO: check_function(self, ..)
						self.params.clone(),
						self.ret,
					),
					def: sir::Func {
						locals: HashMap::new(),
					},
					errors: vec![Error::new(ErrorKind::MissingBody)],
				};
			}
		};

		self.check_body(body)
	}

	pub fn check_body(self, body: &BlockStmt) -> FunctionCheckerResult<'tcx> {
		for stmt in &body.stmts {
			self.check_stmt(stmt);
		}

		if !matches!(self.ret.kind(), TyKind::Void) && !self.has_returned.get() {
			self.add_error(ErrorKind::UnexpectedVoid);
		}

		FunctionCheckerResult {
			ty: crate::kind::Function::new(
				// TODO: check_function(self, ..)
				self.params.clone(),
				self.ret,
			),
			def: sir::Func {
				locals: self.base.locals.into_inner(),
			},
			errors: self.base.errors.into_inner(),
		}
	}

	pub fn check_stmt(&self, stmt: &Stmt) {
		match stmt {
			Stmt::Return(ReturnStmt { arg, .. }) => {
				let expected = self.ret;

				if let Some(arg) = arg {
					let actual = self.check_expr(arg);

					if !self.satisfies(expected, actual.ty) {
						self.raise_type_error(expected, actual.ty);
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
