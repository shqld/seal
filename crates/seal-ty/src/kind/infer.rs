use std::{cell::Cell, hash::Hash};

use crate::Ty;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Infer<'tcx>(Cell<InferKind<'tcx>>);

impl Hash for Infer<'_> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.get().hash(state);
	}
}

impl<'tcx> Infer<'tcx> {
	pub fn new(id: usize) -> Self {
		Self(Cell::new(InferKind::Unresolved(id)))
	}

	pub fn id(&self) -> usize {
		match self.0.get() {
			InferKind::Unresolved(id) => id,
			InferKind::Resolved(_) => panic!("resolved infer has no id"),
		}
	}

	pub fn kind(&self) -> InferKind<'tcx> {
		self.0.get()
	}

	pub fn is_resolved(&self) -> bool {
		matches!(self.0.get(), InferKind::Resolved(_))
	}

	pub fn as_resolved(&self) -> Option<Ty<'tcx>> {
		match self.0.get() {
			InferKind::Resolved(ty) => Some(ty),
			_ => None,
		}
	}

	pub fn unify(&self, ty: Ty<'tcx>) {
		self.0.set(InferKind::Resolved(ty));
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum InferKind<'tcx> {
	Unresolved(usize),
	Resolved(Ty<'tcx>),
}
