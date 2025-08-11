use std::ops::Deref;

use base::BaseChecker;
use errors::{Error, ErrorKind};
use swc_ecma_ast::{ModuleItem, Program, Stmt};

use crate::context::TyContext;

mod base;
mod class;
pub mod errors;
pub mod expr;
mod function;

#[derive(Debug)]
pub struct Checker<'tcx> {
	base: BaseChecker<'tcx>,
}

impl<'tcx> Deref for Checker<'tcx> {
	type Target = BaseChecker<'tcx>;

	fn deref(&self) -> &Self::Target {
		&self.base
	}
}

impl<'tcx> Checker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> Checker<'tcx> {
		Checker {
			base: BaseChecker::new(tcx),
		}
	}

	pub fn check(self, ast: &Program) -> Result<(), Vec<Error<'tcx>>> {
		match &ast {
			Program::Script(script) => {
				for stmt in &script.body {
					self.check_stmt(stmt);
				}
			}
			Program::Module(module) => {
				for module_item in &module.body {
					match module_item {
						ModuleItem::Stmt(stmt) => self.check_stmt(stmt),
						_ => todo!("{:#?}", module_item),
					}
				}
			}
		};

		let errors = self.base.errors.into_inner();

		if errors.is_empty() {
			Ok(())
		} else {
			Err(errors)
		}
	}

	pub fn check_stmt(&self, stmt: &Stmt) {
		match stmt {
			Stmt::Return(return_stmt) => {
				self.add_error_with_span(ErrorKind::UnexpectedReturn, return_stmt.span);
			}
			_ => self.base.check_stmt(stmt),
		}
	}
}
