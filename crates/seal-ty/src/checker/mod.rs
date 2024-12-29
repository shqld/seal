pub mod check;
pub mod parse;
mod satisfies;

use crate::{Ty, TyKind, context::TyContext, type_builder::TypeBuilder};

pub struct TypeChecker<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
	_ty_builder: TypeBuilder<'tcx>,
}

impl<'tcx> TypeChecker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> TypeChecker<'tcx> {
		let _ty_builder = TypeBuilder::new(tcx);
		TypeChecker { tcx, _ty_builder }
	}

	pub fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		self.tcx.new_ty(kind)
	}
}
