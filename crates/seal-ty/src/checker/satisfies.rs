use crate::Ty;

use super::TypeChecker;

impl<'tcx> TypeChecker<'tcx> {
	pub fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
		use crate::TyKind::*;

		match (expected.kind(), actual.kind()) {
			(Infer(id), _) => match self.tcx.infer.resolve_ty(*id) {
				Some(expected) => self.satisfies(expected, actual),
				None => {
					panic!("Expecting unresolved infer type");
				}
			},
			(_, Infer(id)) => match self.tcx.infer.resolve_ty(*id) {
				Some(actual) => self.satisfies(expected, actual),
				None => {
					self.tcx.infer.add_constraint(*id, expected);
					// TODO: unify when function scope ends
					self.tcx.infer.unify(*id, expected);

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
