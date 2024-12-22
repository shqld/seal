use std::{cell::RefCell, collections::HashMap};

use swc_ecma_ast::Id;

use crate::{Ty, TyKind, infer::InferContext, interner::interner::Interner};

pub struct TyContext<'tcx> {
	interner: Interner<'tcx, TyKind<'tcx>>,
	map: RefCell<HashMap<Id, Ty<'tcx>>>,

	pub infer: InferContext<'tcx>,
}

impl<'tcx> TyContext<'tcx> {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			interner: Interner::new(),
			infer: InferContext::new(),
			map: RefCell::new(HashMap::new()),
		}
	}

	pub fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		Ty::new(self.interner.intern(kind))
	}

	pub fn new_infer_ty(&'tcx self) -> Ty<'tcx> {
		let id = self.infer.new_id();
		Ty::new(self.interner.intern(TyKind::Infer(id)))
	}

	pub fn get_ty(&'tcx self, id: &Id) -> Option<Ty<'tcx>> {
		self.map.borrow().get(id).cloned()
	}

	pub fn set_ty(&'tcx self, id: Id, ty: Ty<'tcx>) {
		self.map.borrow_mut().insert(id, ty);
	}
}
