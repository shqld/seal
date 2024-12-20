pub mod infer;

use std::{fmt::Display, hash::Hash};

use crate::Ty;

use self::infer::{Infer, InferKind};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum TyKind<'tcx> {
	Boolean,
	Number,
	String,
	Err,
	Infer(Infer<'tcx>),
	Function {
		params: Vec<Ty<'tcx>>,
		ret: Ty<'tcx>,
	},
	Void,
}

impl Display for TyKind<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TyKind::Boolean => write!(f, "boolean"),
			TyKind::Number => write!(f, "number"),
			TyKind::String => write!(f, "string"),
			TyKind::Err => write!(f, "<err>"),
			TyKind::Infer(inf) => match inf.kind() {
				InferKind::Resolved(ty) => write!(f, "{}", ty),
				InferKind::Unresolved(id) => write!(f, "<infer: {}>", id),
			},
			TyKind::Function { params, ret } => write!(
				f,
				"({}) => {}",
				params
					.iter()
					.map(|ty| ty.to_string())
					.collect::<Vec<_>>()
					.join(", "),
				ret
			),
			TyKind::Void => write!(f, "void"),
		}
	}
}

impl<'tcx> TyKind<'tcx> {
	pub fn is_err(&self) -> bool {
		matches!(self, TyKind::Err)
	}

	pub fn as_infer(&self) -> Option<&Infer<'tcx>> {
		match self {
			TyKind::Infer(infer) => Some(infer),
			_ => None,
		}
	}
}
