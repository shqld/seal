use std::fmt::Display;

use swc_atoms::Atom;
use swc_common::Span;

use crate::{Ty, TyKind, symbol::Symbol};

#[derive(Debug)]
pub struct Error<'tcx> {
	pub kind: ErrorKind<'tcx>,
	pub span: Span,
}

impl<'tcx> Error<'tcx> {
	pub fn new(kind: ErrorKind<'tcx>, span: Span) -> Self {
		Self { kind, span }
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
	/// TS(1196)
	CatchParameterCannotHaveTypeAnnotation,

	/// TS(2304)
	CannotFindName(Symbol),
	/// TS(2322)
	NotAssignable(Ty<'tcx>, Ty<'tcx>),
	/// TS(2324)
	PropertyMissing(Atom, Ty<'tcx>),
	/// TS(2326)
	PropertyTypesIncompatible(Atom, Ty<'tcx>, Ty<'tcx>),
	/// TS(2339)
	PropertyDoesNotExist(Ty<'tcx>, Atom),
	/// TS(2345)
	ArgumentNotAssignable(Ty<'tcx>, Ty<'tcx>),
	/// TS(2349)
	NotCallable(Ty<'tcx>),
	/// TS(2351)
	NotConstructable,
	/// TS(2355)
	UnexpectedVoid,
	/// TS(2367)
	NoOverlap(Ty<'tcx>, Ty<'tcx>),
	/// TS(2454)
	UsedBeforeAssigned(Symbol),
	/// TS(2538)
	TypeCannotBeUsedAsIndexType(Ty<'tcx>),
	/// TS(2540)
	CannotAssignToReadOnlyProperty(Atom),
	/// TS(2554)
	WrongNumArgs(usize, usize),
	/// TS(2588)
	CannotAssignToConst(Symbol),
	/// Custom error for binary operator type mismatch
	BinaryOperatorTypeMismatch(swc_ecma_ast::BinaryOp, Ty<'tcx>, Ty<'tcx>),
	/// Custom error for extending non-class type
	ExtendsNonClass(Ty<'tcx>),
	InvalidNumberLiteral(f64),
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

			// TS(2304)
			CannotFindName(name) => {
				write!(f, "Cannot find name '{}'.", name)
			}
			// TS(1155)
			ConstMissingInit => {
				write!(f, "'const' declarations must be initialized.")
			}
			// TS(2322)
			NotAssignable(expected, actual) => {
				// Handle literal string types specially
				if !matches!(expected.kind(), String(_)) && matches!(actual.kind(), String(Some(_)))
				{
					write!(f, "Type 'string' is not assignable to type '{expected}'.")
				} else if !matches!(expected.kind(), Number(_))
					&& matches!(actual.kind(), Number(Some(_)))
				{
					write!(f, "Type 'number' is not assignable to type '{expected}'.")
				} else if !matches!(expected.kind(), Boolean(_))
					&& matches!(actual.kind(), Boolean(Some(_)))
				{
					write!(f, "Type 'boolean' is not assignable to type '{expected}'.")
				} else {
					write!(f, "Type '{actual}' is not assignable to type '{expected}'.")
				}
			}
			// TS(2324)
			PropertyMissing(prop, ty) => {
				write!(f, "Property '{prop}' is missing in type '{ty}'.")
			}
			// TS(2326)
			PropertyTypesIncompatible(prop, expected, actual) => {
				write!(
					f,
					"Types of property '{prop}' are incompatible.\n  Type '{actual}' is not assignable to type '{expected}'."
				)
			}
			// TS(2355)
			UnexpectedVoid => {
				write!(
					f,
					"A function whose declared type is 'void' must return a value."
				)
			}
			// TS(2367)
			NoOverlap(left, right) => {
				write!(
					f,
					"This comparison appears to be unintentional because the types '{left}' and '{right}' have no overlap."
				)
			}
			// TS(2339)
			PropertyDoesNotExist(ty, key) => {
				write!(f, "Property '{key}' does not exist on type '{ty}'.")
			}
			// TS(2345)
			ArgumentNotAssignable(expected, actual) => {
				write!(
					f,
					"Argument of type '{actual}' is not assignable to parameter of type '{expected}'."
				)
			}
			// TS(2351)
			NotConstructable => {
				write!(f, "This expression is not constructable.")
			}
			// TS(2554)
			WrongNumArgs(expected, actual) => {
				write!(f, "Expected {expected} arguments, but got {actual}.")
			}
			// TS(1108)
			UnexpectedReturn => {
				write!(
					f,
					"A 'return' statement can only be used within a function body."
				)
			}
			// TS(2588)
			CannotAssignToConst(name) => {
				write!(f, "Cannot assign to '{name}' because it is a constant.")
			}
			// TS(2349)
			NotCallable(ty) => {
				write!(
					f,
					"This expression is not callable.\n  Type '{ty}' has no call signatures."
				)
			}
			// TS(2454)
			UsedBeforeAssigned(name) => {
				write!(f, "Variable '{name}' is used before being assigned.")
			}
			// TS(2538)
			TypeCannotBeUsedAsIndexType(ty) => {
				write!(f, "Type '{ty}' cannot be used as an index type.")
			}
			// TS(2540)
			CannotAssignToReadOnlyProperty(prop) => {
				write!(
					f,
					"Cannot assign to '{prop}' because it is a read-only property."
				)
			}
			// TS(1196)
			CatchParameterCannotHaveTypeAnnotation => {
				write!(f, "Catch clause parameter cannot have a type annotation.")
			}
			// Custom errors
			BinaryOperatorTypeMismatch(op, left, right) => {
				let op_str = match op {
					swc_ecma_ast::BinaryOp::Add => "+",
					swc_ecma_ast::BinaryOp::Sub => "-",
					swc_ecma_ast::BinaryOp::Mul => "*",
					swc_ecma_ast::BinaryOp::Div => "/",
					swc_ecma_ast::BinaryOp::Lt => "<",
					swc_ecma_ast::BinaryOp::LtEq => "<=",
					swc_ecma_ast::BinaryOp::Gt => ">",
					swc_ecma_ast::BinaryOp::GtEq => ">=",
					_ => "unknown",
				};
				write!(
					f,
					"Operator '{op_str}' cannot be applied to types '{left}' and '{right}'."
				)
			}
			ExtendsNonClass(ty) => {
				write!(
					f,
					"Class extends value '{ty}' which is not a constructor function type."
				)
			}
			InvalidNumberLiteral(value) => {
				write!(f, "Invalid number literal: {value}.")
			}
		}
	}
}
