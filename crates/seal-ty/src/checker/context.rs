use std::{cell::RefCell, collections::HashMap};

use swc_ecma_ast::{Id, Program};

use crate::{Ty, TyKind, interner::interner::Interner};

pub struct TyContext<'tcx> {
	pub ast: Program,
	interner: Interner<'tcx, TyKind<'tcx>>,
	map: RefCell<HashMap<Id, Ty<'tcx>>>,
}

impl<'tcx> TyContext<'tcx> {
	#[allow(clippy::new_without_default)]
	pub fn new(ast: Program) -> Self {
		Self {
			ast,
			interner: Interner::new(),
			map: RefCell::new(HashMap::new()),
		}
	}

	pub fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		Ty::new(self.interner.intern(kind))
	}

	pub fn get_ty(&'tcx self, id: &Id) -> Option<Ty<'tcx>> {
		self.map.borrow().get(id).cloned()
	}

	pub fn set_ty(&'tcx self, id: Id, ty: Ty<'tcx>) {
		self.map.borrow_mut().insert(id, ty);
	}
}
