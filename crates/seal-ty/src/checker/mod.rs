pub mod check;
pub mod context;
// pub mod infer;
pub mod parse;
pub mod scope;

use std::{cell::RefCell, rc::Rc};

use context::TyContext;
use scope::Scope;
use swc_common::SyntaxContext;

pub struct Checker<'tcx> {
	tcx: TyContext<'tcx>,
	scopes: RefCell<Vec<Rc<Scope>>>,
}

impl<'tcx> Checker<'tcx> {
	pub fn new(tcx: TyContext<'tcx>) -> Checker<'tcx> {
		Checker {
			tcx,
			scopes: RefCell::new(vec![]),
		}
	}

	fn push_scope(&self, ctx: SyntaxContext) -> Rc<Scope> {
		let scope = Rc::new(Scope::new(ctx));

		self.scopes.borrow_mut().push(scope.clone());

		scope
	}

	fn get_current_scope(&self) -> Rc<Scope> {
		self.scopes.borrow().last().unwrap().clone()
	}

	fn pop_scope(&self) {
		self.scopes.borrow_mut().pop();
	}
}
