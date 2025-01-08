use std::{
	fmt::{Debug, Display},
	ops::Deref,
};

use crate::{intern::interned::Interned, kind::TyKind};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ty<'tcx>(Interned<'tcx, TyKind<'tcx>>);

impl<'tcx> Deref for Ty<'tcx> {
	type Target = TyKind<'tcx>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Display for Ty<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl<'tcx> Ty<'tcx> {
	pub fn new(interned: Interned<'tcx, TyKind<'tcx>>) -> Self {
		Self(interned)
	}

	pub fn kind(&self) -> &TyKind<'tcx> {
		&self.0
	}

	pub fn update(&mut self, ty: Ty<'tcx>) -> Self {
		self.0 = ty.0;
		*self
	}
}
