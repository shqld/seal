pub mod check;
pub mod parse;
mod satisfies;

use std::cell::RefCell;

use crate::{Ty, TyKind, constants::TyConstants, context::TyContext};

pub struct TypeChecker<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
	errors: RefCell<Vec<String>>,
	constants: TyConstants<'tcx>,
}

impl<'tcx> TypeChecker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> TypeChecker<'tcx> {
		TypeChecker {
			tcx,
			errors: RefCell::new(vec![]),
			constants: TyConstants::new(tcx),
		}
	}

	pub fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		self.tcx.new_ty(kind)
	}

	pub fn add_error(&self, error: String) {
		self.errors.borrow_mut().push(error);
	}
}
