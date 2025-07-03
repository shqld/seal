use crate::Ty;

use super::BaseChecker;

impl<'tcx> BaseChecker<'tcx> {
	#[allow(clippy::only_used_in_recursion)]
	pub fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
		use crate::TyKind::*;

		match (expected.kind(), actual.kind()) {
			// to prevent cascading errors
			(Err, _) | (_, Err) => true,
			// never type is the bottom type - nothing can be assigned to it except never itself
			(Never, Never) => true,
			(Never, _) => false,
			(_, Never) => true, // never can be assigned to anything
			// unknown type is the top type - anything can be assigned to it
			(Unknown, _) => true,
			// null type is distinct
			(Null, Null) => true,
			// Lazy types must be replaced with their actual types before checking
			(Lazy, _) | (_, Lazy) => panic!("Lazy types must not be present in satisfies"),
			// any guard can satisfy 'boolean', not vice versa
			(Boolean, Guard(_, _)) => true,

			// any const string can satisfy 'string', not vice versa
			(String(None), String(_)) => true,

			(Function(expected), Function(actual)) => {
				if expected.params.len() != actual.params.len() {
					return false;
				}

				if !self.satisfies(expected.ret, actual.ret) {
					return false;
				}

				for ((_, expected), (_, actual)) in expected.params.iter().zip(&actual.params) {
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

			(Array(expected), Array(actual)) => self.satisfies(expected.element, actual.element),

			// Object literal should satisfy Object type
			(Interface(expected), Object(_)) if expected.name().name() == "Object" => true,

			_ => expected == actual,
		}
	}

	pub fn overlaps(&self, left: Ty<'tcx>, right: Ty<'tcx>) -> bool {
		self.satisfies(left, right) || self.satisfies(right, left)
	}
}
