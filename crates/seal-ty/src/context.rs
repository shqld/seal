use std::{cell::RefCell, collections::HashMap};

use crate::{Ty, TyKind, builder::sir::Symbol, infer::InferContext, interner::interner::Interner};

#[derive(Debug)]
pub struct TyContext<'tcx> {
	pub interner: Interner<'tcx, TyKind<'tcx>>,
	pub types: RefCell<HashMap<Symbol, Ty<'tcx>>>,
	pub infer: InferContext<'tcx>,
}

impl<'tcx> TyContext<'tcx> {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			interner: Interner::new(),
			infer: InferContext::new(),
			types: RefCell::new(HashMap::new()),
		}
	}

	pub fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		Ty::new(self.interner.intern(kind))
	}

	pub fn new_infer_ty(&'tcx self) -> Ty<'tcx> {
		let id = self.infer.new_id();
		Ty::new(self.interner.intern(TyKind::Infer(id)))
	}

	pub fn get_ty(&self, id: &Symbol) -> Option<Ty<'tcx>> {
		self.types.borrow().get(id).cloned()
	}

	pub fn set_ty(&self, id: Symbol, ty: Ty<'tcx>) {
		self.types.borrow_mut().insert(id, ty);
	}
}
