use std::{
	collections::{BTreeMap, BTreeSet},
	fmt::Display,
	hash::Hash,
};

use swc_atoms::Atom;

use crate::{Ty, builder::sir::Symbol, infer::InferId};

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
	Object(Object<'tcx>),
	Guard(Symbol, Ty<'tcx>),
}

impl Display for TyKind<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TyKind::Boolean | TyKind::Guard(_, _) => write!(f, "boolean"),
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
			TyKind::Union(Union { arms: tys }) => write!(
				f,
				"{}",
				tys.iter()
					.map(|ty| ty.to_string())
					.collect::<Vec<_>>()
					.join(" | ")
			),
			TyKind::Never => write!(f, "never"),
			TyKind::Object(Object { fields }) => write!(
				f,
				"{{{}}}",
				fields
					.iter()
					.map(|(name, ty)| format!("{}: {}", name, ty))
					.collect::<Vec<_>>()
					.join(", ")
			),
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
	arms: BTreeSet<Ty<'tcx>>,
}

impl<'tcx> Union<'tcx> {
	pub fn new(tys: BTreeSet<Ty<'tcx>>) -> Self {
		assert!(tys.len() >= 2);

		let mut inner = BTreeSet::new();

		for ty in tys {
			match ty.kind() {
				TyKind::Union(union) => {
					for ty in union.arms() {
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

		Self { arms: inner }
	}

	pub fn arms(&self) -> &BTreeSet<Ty<'tcx>> {
		&self.arms
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Object<'tcx> {
	fields: BTreeMap<Atom, Ty<'tcx>>,
}

impl<'tcx> Object<'tcx> {
	pub fn new(fields: BTreeMap<Atom, Ty<'tcx>>) -> Self {
		Self { fields }
	}

	pub fn fields(&self) -> &BTreeMap<Atom, Ty<'tcx>> {
		&self.fields
	}

	pub fn get_prop(&self, prop: &Atom) -> Option<Ty<'tcx>> {
		self.fields.get(prop).copied()
	}
}
