use std::{fmt::Display, hash::Hash};

use crate::{Ty, infer::InferId};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum TyKind<'tcx> {
	Boolean,
	Number,
	String,
	Err,
	Function(FunctionTy<'tcx>),
	Void,
	Infer(InferId),
}

impl Display for TyKind<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TyKind::Boolean => write!(f, "boolean"),
			TyKind::Number => write!(f, "number"),
			TyKind::String => write!(f, "string"),
			TyKind::Err => write!(f, "<err>"),
			TyKind::Infer(id) => write!(f, "<infer: {id}>",),
			TyKind::Function(FunctionTy { params, ret }) => write!(
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

impl TyKind<'_> {
	pub fn is_err(&self) -> bool {
		matches!(self, TyKind::Err)
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct FunctionTy<'tcx> {
	pub params: Vec<Ty<'tcx>>,
	pub ret: Ty<'tcx>,
}
