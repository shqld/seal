use std::{
	cell::{Cell, RefCell},
	collections::{HashMap, hash_map::Entry},
	fmt::{Debug, Display},
	hash::Hash,
};

use crate::Ty;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InferId(usize);

impl Display for InferId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self.0, f)
	}
}

pub struct InferContext<'tcx> {
	// TODO: infer_constraints
	constraints: RefCell<Vec<(InferId, Ty<'tcx>)>>,
	infer_count: Cell<usize>,
	map: RefCell<HashMap<InferId, Ty<'tcx>>>,
}

impl<'tcx> InferContext<'tcx> {
	pub fn new() -> Self {
		Self {
			constraints: RefCell::new(Vec::new()),
			infer_count: Cell::new(0),
			map: RefCell::new(HashMap::new()),
		}
	}

	pub fn new_id(&self) -> InferId {
		let id = self.infer_count.get();
		self.infer_count.set(id + 1);

		InferId(id)
	}

	pub fn add_constraint(&self, id: InferId, ty: Ty<'tcx>) {
		self.constraints.borrow_mut().push((id, ty));
	}

	pub fn unify(&self, id: InferId, ty: Ty<'tcx>) {
		let mut map = self.map.borrow_mut();

		if let Entry::Vacant(e) = map.entry(id) {
			e.insert(ty);
		} else {
			panic!("Infer type already resolved: {:?}", id);
		}
	}

	pub fn resolve_ty(&self, id: InferId) -> Option<Ty<'tcx>> {
		self.map.borrow().get(&id).copied()
	}
}

impl Default for InferContext<'_> {
	fn default() -> Self {
		Self::new()
	}
}
