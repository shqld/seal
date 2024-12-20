pub mod check;
pub mod context;
pub mod parse;

use std::cell::RefCell;

use context::TyContext;

use crate::Ty;

pub struct Checker<'tcx> {
	tcx: TyContext<'tcx>,
	function_scopes: RefCell<Vec<FunctionScope<'tcx>>>,
}

impl<'tcx> Checker<'tcx> {
	pub fn new(tcx: TyContext<'tcx>) -> Checker<'tcx> {
		Checker {
			tcx,
			function_scopes: RefCell::new(vec![]),
		}
	}

	fn push_function_scope(&self, return_ty: Ty<'tcx>) {
		self.function_scopes.borrow_mut().push(FunctionScope {
			return_ty,
			has_returned: false,
		});
	}

	fn set_function_has_returned(&self) {
		self.function_scopes
			.borrow_mut()
			.last_mut()
			.unwrap()
			.has_returned = true;
	}

	fn get_current_function_return_ty(&self) -> Option<Ty<'tcx>> {
		self.function_scopes
			.borrow()
			.last()
			.map(|scope| scope.return_ty)
	}

	fn get_current_function_has_returned(&self) -> bool {
		self.function_scopes
			.borrow()
			.last()
			.map(|scope| scope.has_returned)
			.unwrap_or(false)
	}

	fn pop_function_scope(&self) {
		self.function_scopes.borrow_mut().pop();
	}
}

struct FunctionScope<'tcx> {
	return_ty: Ty<'tcx>,
	// TODO: cfg
	has_returned: bool,
}
