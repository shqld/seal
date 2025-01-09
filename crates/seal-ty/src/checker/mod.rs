use std::ops::Deref;

use base::BaseChecker;
use swc_ecma_ast::{ModuleItem, Program, Stmt};

use crate::context::TyContext;

mod base;
mod function;
mod scope;

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

	pub fn check(self, ast: &Program) -> Result<(), Vec<String>> {
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
						_ => unimplemented!("{:#?}", module_item),
					}
				}
			}
		};

		self.base.into_result()
	}

	pub fn check_stmt(&self, stmt: &Stmt) {
		match stmt {
			Stmt::Return(_) => {
				// TS(1108)
				panic!("Return statement is not allowed in the main function");
			}
			_ => self.base.check_stmt(stmt),
		}
	}
}
