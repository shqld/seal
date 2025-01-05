use swc_atoms::Atom;

use crate::{Ty, context::TyContext};

#[derive(Debug)]
pub struct TyConstants<'tcx> {
	pub type_of: Ty<'tcx>,
}

impl<'tcx> TyConstants<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> Self {
		Self {
			type_of: tcx.new_union(
				["boolean", "number", "string"]
					.iter()
					.map(|s| tcx.new_const_string(Atom::new(*s)))
					.collect(),
			),
		}
	}
}
