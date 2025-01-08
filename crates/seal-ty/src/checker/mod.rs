use std::ops::Deref;

use base::BaseChecker;
use function::FunctionChecker;
use swc_ecma_ast::{Decl, FnDecl, ModuleItem, Pat, Program, Stmt};

use crate::{context::TyContext, symbol::Symbol};

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
			Stmt::Decl(decl) => self.check_decl(decl),
			Stmt::Return(_) => {
				// TS(1108)
				panic!("Return statement is not allowed in the main function");
			}
			_ => self.base.check_stmt(stmt),
		}
	}

	pub fn check_decl(&self, decl: &Decl) {
		match decl {
			Decl::Fn(FnDecl {
				ident, function, ..
			}) => {
				let name = Symbol::new(ident.to_id());

				let mut params = vec![];
				for param in &function.params {
					match &param.pat {
						Pat::Ident(ident) => {
							let name = Symbol::new(ident.to_id());

							let ty = match &ident.type_ann {
								Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
								None => {
									panic!("Param type annotation is required");
								}
							};

							params.push((name, ty));
						}
						_ => unimplemented!("{:#?}", param),
					}
				}

				let ret = match &function.return_type {
					Some(type_ann) => self.build_ts_type(&type_ann.type_ann),
					None => {
						// NOTE: seal does't infer the return type
						self.constants.void
					}
				};

				let checker = FunctionChecker::new(self.tcx, &params, ret);

				checker.check_function(function);

				if let Err(errors) = checker.into_result() {
					for error in errors {
						self.add_error(error);
					}
				};

				self.add_var(&name, self.tcx.new_function(params, ret), false);
			}
			_ => self.base.check_decl(decl),
		}
	}
}
