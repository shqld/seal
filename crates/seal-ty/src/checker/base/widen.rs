use crate::{Ty, TyKind};

use super::BaseChecker;

impl<'tcx> BaseChecker<'tcx> {
	pub fn widen(&self, ty: Ty<'tcx>) -> Ty<'tcx> {
		use TyKind::*;

		match ty.kind() {
			String(_) => self.constants.string,
			Number(_) => self.constants.number,
			Boolean(_) => self.constants.boolean,
			Guard(_, _) => self.constants.boolean,
			_ => ty,
		}
	}
}
