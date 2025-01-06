use crate::Ty;

use super::TypeChecker;

impl<'tcx> TypeChecker<'tcx> {
	pub fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
		use crate::TyKind::*;

		match (expected.kind(), actual.kind()) {
			// prevent cascading errors
			(Err, _) => true,
			// e.g. `let n; n = 1;`
			(Infer(id), _) => match self.tcx.infer.resolve_ty(*id) {
				Some(expected) => self.satisfies(expected, actual),
				None => {
					self.tcx.infer.add_constraint(*id, expected);

					true
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
				if expected.params.len() != actual.params.len() {
					return false;
				}

				if !self.satisfies(expected.ret, actual.ret) {
					return false;
				}

				for (expected, actual) in expected.params.iter().zip(&actual.params) {
					if !self.satisfies(*expected, *actual) {
						return false;
					}
				}

				true
			}
			(Union(expected), Union(actual)) => actual.arms().iter().all(|actual| {
				expected
					.arms()
					.iter()
					.any(|expected| self.satisfies(*expected, *actual))
			}),
			(Union(expected), _) => expected.arms().iter().any(|ty| self.satisfies(*ty, actual)),
			(String(None), String(_)) => true,
			(Boolean, Guard(_, _)) | (Guard(_, _), Boolean) => true,
			_ => expected == actual,
		}
	}

	pub fn overlaps(&self, left: Ty<'tcx>, right: Ty<'tcx>) -> bool {
		self.satisfies(left, right) || self.satisfies(right, left)
	}
}
