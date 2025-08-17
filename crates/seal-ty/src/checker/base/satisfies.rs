use crate::Ty;

use super::BaseChecker;

impl<'tcx> BaseChecker<'tcx> {
	#[allow(clippy::only_used_in_recursion)]
	pub fn satisfies(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) -> bool {
		use crate::TyKind::*;

		let result = match (expected.kind(), actual.kind()) {
			// to prevent cascading errors
			(Err, _) | (_, Err) => true,
			// never type is the bottom type - nothing can be assigned to it except never itself
			(Never, Never) => true,
			(Never, _) | (_, Never) => false,
			// unknown type is the top type - anything can be assigned to it
			(Unknown, _) => true,
			// null type is distinct
			(Null, Null) => true,
			// Lazy types must be replaced with their actual types before checking
			(Lazy, _) | (_, Lazy) => panic!("Lazy types must not be present in satisfies"),
			// any guard can satisfy 'boolean', not vice versa
			(Boolean(_), Guard(_, _)) => true,

			// Literal type assignability rules (TypeScript-compliant):
			// - any const string can satisfy 'string', not vice versa
			(String(None), String(_)) => true,
			// - any const number can satisfy 'number', not vice versa
			(Number(None), Number(_)) => true,
			// - any const boolean can satisfy 'boolean', not vice versa
			(Boolean(None), Boolean(_)) => true,

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
			(_, Union(actual)) => actual.arms().iter().all(|ty| self.satisfies(expected, *ty)),

			(Array(expected), Array(actual)) => self.satisfies(expected.element, actual.element),

			// Tuple type compatibility
			(Tuple(expected), Tuple(actual)) => {
				// Tuples must have same length and each element must satisfy
				if expected.elements.len() != actual.elements.len() {
					return false;
				}

				for (expected_elem, actual_elem) in
					expected.elements.iter().zip(actual.elements.iter())
				{
					if !self.satisfies(*expected_elem, *actual_elem) {
						return false;
					}
				}

				true
			}

			// Object literal should satisfy interface through structural typing
			(Interface(expected), Object(actual_obj)) => {
				// Check if object literal has all properties of interface
				for (prop, expected_ty) in expected.fields() {
					match actual_obj.get_prop(prop) {
						Some(actual_ty) => {
							if !self.satisfies(*expected_ty, actual_ty) {
								return false;
							}
						}
						None => return false,
					}
				}
				true
			}

			// Interface inheritance checking
			(Interface(expected_interface), Interface(actual_interface)) => {
				// First check if names match - this handles exact type matches
				if expected_interface.name() == actual_interface.name() {
					return true;
				}

				// For different interface names, use structural typing:
				// Check if actual interface has all properties of expected interface
				for (prop, expected_ty) in expected_interface.fields() {
					match actual_interface.get_prop(prop) {
						Some(actual_ty) => {
							if !self.satisfies(*expected_ty, actual_ty) {
								return false;
							}
						}
						None => return false,
					}
				}

				// If expected interface has no properties, the structural check would pass
				// but we should only allow this if the interfaces are related through inheritance
				// For now, we'll be strict and require same names for empty interfaces
				if expected_interface.fields().is_empty() && actual_interface.fields().is_empty() {
					// Both are empty but different names - only allow if there's an inheritance relationship
					// For simplicity, we'll return false to maintain existing behavior
					false
				} else {
					true
				}
			}

			// Class inheritance checking
			(Class(_), Class(actual_class)) => {
				// Check if actual is same as expected
				if expected == actual {
					return true;
				}
				// Check if actual extends expected (walk up the inheritance chain)
				let mut current = actual_class.parent();
				// TODO: each class ty should have a set referencing to all ancestors
				while let Some(parent_ty) = current {
					if parent_ty == expected {
						return true;
					}
					// Continue up the chain if parent is also a class
					current = match parent_ty.kind() {
						Class(parent_class) => parent_class.parent(),
						_ => None,
					};
				}
				false
			}

			// Instance of class satisfies parent class
			(Class(expected_class), Interface(actual_interface)) => {
				// An interface satisfies a class if it has all the properties of that class
				for (prop, expected_ty) in expected_class.fields() {
					match actual_interface.get_prop(prop) {
						Some(actual_ty) => {
							if !self.satisfies(*expected_ty, actual_ty) {
								return false;
							}
						}
						None => return false,
					}
				}
				true
			}

			// Object structural compatibility with excess property checking
			(Object(expected_obj), Object(actual_obj)) => {
				if expected_obj.fields.len() != actual_obj.fields.len() {
					return false;
				}

				// For different interface names, use structural typing:
				// Check if actual interface has all properties of expected interface
				for (prop, expected_ty) in expected_obj.fields() {
					match actual_obj.get_prop(prop) {
						Some(actual_ty) => {
							if !self.satisfies(*expected_ty, actual_ty) {
								return false;
							}
						}
						None => return false,
					}
				}

				true
			}
			_ => expected == actual,
		};

		result
	}

	pub fn overlaps(&self, left: Ty<'tcx>, right: Ty<'tcx>) -> bool {
		self.satisfies(left, right) || self.satisfies(right, left)
	}
}
