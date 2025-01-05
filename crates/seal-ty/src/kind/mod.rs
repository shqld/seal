use std::{collections::BTreeSet, fmt::Display, hash::Hash};

use swc_atoms::Atom;

use crate::{Ty, infer::InferId};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum TyKind<'tcx> {
	Boolean,
	Number,
	String(Option<Atom>),
	Err,
	Function(Function<'tcx>),
	Void,
	Infer(InferId),
	Union(Union<'tcx>),
	Never,
}

impl Display for TyKind<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TyKind::Boolean => write!(f, "boolean"),
			TyKind::Number => write!(f, "number"),
			TyKind::String(value) => match value {
				Some(value) => write!(f, "\"{}\"", value),
				None => write!(f, "string"),
			},
			TyKind::Err => write!(f, "<err>"),
			TyKind::Infer(id) => write!(f, "<infer: {id}>",),
			TyKind::Function(Function { params, ret }) => write!(
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
			TyKind::Union(Union { tys }) => write!(
				f,
				"{}",
				tys.iter()
					.map(|ty| ty.to_string())
					.collect::<Vec<_>>()
					.join(" | ")
			),
			TyKind::Never => write!(f, "never"),
		}
	}
}

impl TyKind<'_> {
	pub fn is_err(&self) -> bool {
		matches!(self, TyKind::Err)
	}

	pub fn is_void(&self) -> bool {
		matches!(self, TyKind::Void)
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Function<'tcx> {
	pub params: Vec<Ty<'tcx>>,
	pub ret: Ty<'tcx>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Union<'tcx> {
	tys: BTreeSet<Ty<'tcx>>,
}

impl<'tcx> Union<'tcx> {
	pub fn new(tys: BTreeSet<Ty<'tcx>>) -> Self {
		assert!(tys.len() >= 2);

		let mut inner = BTreeSet::new();

		for ty in tys {
			match ty.kind() {
				TyKind::Union(union) => {
					for ty in union.tys() {
						inner.insert(*ty);
					}
				}
				TyKind::Infer(_) => {
					panic!("Union type cannot contain infer types");
				}
				_ => {
					inner.insert(ty);
				}
			}
		}

		Self { tys: inner }
	}

	pub fn tys(&self) -> &BTreeSet<Ty<'tcx>> {
		&self.tys
	}
}
