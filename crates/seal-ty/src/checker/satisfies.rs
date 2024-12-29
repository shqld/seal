use crate::Ty;

use super::TypeChecker;

impl<'tcx> TypeChecker<'tcx> {
	pub fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
		use crate::TyKind::*;

		match (expected.kind(), actual.kind()) {
			// e.g. function return type
			(Infer(id), _) => match self.tcx.infer.resolve_ty(*id) {
				Some(expected) => self.satisfies(expected, actual),
				None => {
					panic!("Expecting unresolved infer type");
				}
			},
			// e.g. function param types
			(_, Infer(id)) => match self.tcx.infer.resolve_ty(*id) {
				Some(actual) => self.satisfies(expected, actual),
				None => {
					self.tcx.infer.add_constraint(*id, expected);

					true
				}
			},
			(Function(expected), Function(actual)) => {
				for (expected, actual) in expected.params.iter().zip(&actual.params) {
					if !self.satisfies(*expected, *actual) {
						return false;
					}
				}

				self.satisfies(expected.ret, actual.ret)
			}
			_ => expected == actual,
		}
	}
}
