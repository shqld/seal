use std::fmt::Display;

use swc_atoms::Atom;

use crate::{Ty, TyKind, symbol::Symbol};

#[derive(Debug)]
pub struct Error<'tcx> {
	pub kind: ErrorKind<'tcx>,
}

impl<'tcx> Error<'tcx> {
	pub fn new(kind: ErrorKind<'tcx>) -> Self {
		Self { kind }
	}
}

impl Display for Error<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.kind)
	}
}

#[derive(Debug)]
pub enum ErrorKind<'tcx> {
	ClassPropMissingTypeAnnOrInit,
	ClassCtorWithReturn,
	NewOpMissingArgs,
	ParamMissingTypeAnn,
	MissingBody,
	Var,

	/// TS(1108)
	UnexpectedReturn,
	/// TS(1155):
	ConstMissingInit,

	/// TS(2304)
	CannotFindName(Symbol),
	/// TS(2322)
	NotAssignable(Ty<'tcx>, Ty<'tcx>),
	/// TS(2339)
	PropertyDoesNotExist(Ty<'tcx>, Atom),
	/// TS(2351)
	NotConstructable,
	/// TS(2454)
	UsedBeforeAssigned(Symbol),
	/// TS(2554)
	WrongNumArgs(usize, usize),
	/// TS(2588)
	CannotAssignToConst(Symbol),
	/// TS(2355)
	UnexpectedVoid,
	// TS(2349)
	NotCallable(Ty<'tcx>),

	/// TS(2367)
	NoOverlap(Ty<'tcx>, Ty<'tcx>),
	/// TS(1196)
	CatchParameterCannotHaveTypeAnnotation,
}

impl Display for ErrorKind<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use ErrorKind::*;
		use TyKind::*;

		match self {
			ClassPropMissingTypeAnnOrInit => {
				write!(f, "Type annotation or initializer is required.")
			}
			ClassCtorWithReturn => {
				write!(f, "Constructor cannot have return statement.")
			}
			NewOpMissingArgs => {
				write!(f, "Arguments must follow after 'new'.")
			}
			ParamMissingTypeAnn => {
				write!(f, "Parameter must have a type annotation.")
			}
			MissingBody => {
				write!(f, "Body is required.")
			}
			Var => {
				write!(f, "'var' is not allowed")
			}

			CannotFindName(name) => {
				write!(f, "Cannot find name '{}'.", name)
			}
			ConstMissingInit => {
				write!(f, "'const' declarations must be initialized.")
			}
			NotAssignable(expected, actual) => {
				// e.g. expected: number, actual: "42" -> Type 'string' is not assignable to type 'number'
				if !matches!(expected.kind(), String(_)) && matches!(actual.kind(), String(Some(_)))
				{
					write!(
						f,
						"Type '{actual}' is not assignable to type '{expected}'.",
						actual = TyKind::String(None)
					)
				} else {
					write!(f, "Type '{actual}' is not assignable to type '{expected}'.")
				}
			}
			UnexpectedVoid => {
				write!(
					f,
					"A function whose declared type is 'void' must return a value."
				)
			}
			NoOverlap(left, right) => {
				write!(
					f,
					"This comparison appears to be unintentional because the types '{left}' and '{right}' have no overlap."
				)
			}
			PropertyDoesNotExist(ty, key) => {
				write!(f, "Property '{key}' does not exist on type '{ty}'.")
			}
			NotConstructable => {
				write!(f, "This expression is not constructable.")
			}
			WrongNumArgs(expected, actual) => {
				write!(f, "Expected {expected} arguments, but got {actual}.")
			}
			UnexpectedReturn => {
				write!(
					f,
					"A 'return' statement can only be used within a function body."
				)
			}
			CannotAssignToConst(name) => {
				write!(f, "Cannot assign to '{}' because it is a constant.", name)
			}
			NotCallable(ty) => {
				write!(
					f,
					"This expression is not callable.\nType '{ty}' has no call signatures.",
				)
			}
			UsedBeforeAssigned(name) => {
				write!(f, "Variable '{name}' is used before being assigned.")
			}
			CatchParameterCannotHaveTypeAnnotation => {
				write!(f, "Catch clause parameter cannot have a type annotation.")
			}
		}
	}
}
