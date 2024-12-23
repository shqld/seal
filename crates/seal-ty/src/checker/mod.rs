pub mod check;
pub mod parse;
pub mod scope;

use std::{cell::RefCell, rc::Rc};

use scope::Scope;
use swc_common::SyntaxContext;

use crate::context::TyContext;

pub struct Checker<'tcx> {
	pub tcx: TyContext<'tcx>,
	pub scopes: RefCell<Vec<Rc<Scope>>>,
}

impl Checker<'_> {
	pub fn new(tcx: TyContext) -> Checker {
		Checker {
			tcx,
			scopes: RefCell::new(vec![]),
		}
	}

	pub fn push_scope(&self, ctx: SyntaxContext) -> Rc<Scope> {
		let scope = Rc::new(Scope::new(ctx));

		self.scopes.borrow_mut().push(scope.clone());

		scope
	}

	pub fn get_current_scope(&self) -> Rc<Scope> {
		self.scopes.borrow().last().unwrap().clone()
	}

	pub fn pop_scope(&self) {
		self.scopes.borrow_mut().pop();
	}
}
