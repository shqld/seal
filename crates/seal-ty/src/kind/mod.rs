use std::{
	collections::{BTreeMap, BTreeSet},
	fmt::Display,
	hash::Hash,
};

use swc_atoms::Atom;

use crate::{Ty, symbol::Symbol};

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum TyKind<'tcx> {
	// value types
	Void,
	Boolean,
	Number,
	String(Option<Atom>),
	Object(Object<'tcx>),
	Function(Function<'tcx>),

	// special types
	Union(Union<'tcx>),

	// internal checker types (users cannot create)
	Err,
	Lazy,
	Never,
	Guard(Symbol, Ty<'tcx>),
}

impl Display for TyKind<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TyKind::Void => write!(f, "void"),
			TyKind::Boolean => write!(f, "boolean"),
			TyKind::Number => write!(f, "number"),
			TyKind::String(value) => match value {
				Some(value) => write!(f, "\"{}\"", value),
				None => write!(f, "string"),
			},
			TyKind::Function(Function { params, ret }) => {
				let params = params
					.iter()
					.map(|(name, ty)| format!("{}: {}", name, ty))
					.collect::<Vec<_>>()
					.join(", ");

				write!(f, "({params}) => {ret}")
			}
			TyKind::Object(Object { fields }) => write!(
				f,
				"{{{}}}",
				fields
					.iter()
					.map(|(name, ty)| format!("{}: {}", name, ty))
					.collect::<Vec<_>>()
					.join(", ")
			),
			TyKind::Union(Union { arms: tys }) => write!(
				f,
				"{}",
				tys.iter()
					.map(|ty| ty.to_string())
					.collect::<Vec<_>>()
					.join(" | ")
			),
			TyKind::Err => write!(f, "<err>"),
			TyKind::Lazy => write!(f, "<lazy>",),
			TyKind::Never => write!(f, "<never>",),
			TyKind::Guard(_, _) => write!(f, "<guard>"),
		}
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Function<'tcx> {
	pub params: Vec<(Symbol, Ty<'tcx>)>,
	pub ret: Ty<'tcx>,
}

impl<'tcx> Function<'tcx> {
	pub fn new(params: Vec<(Symbol, Ty<'tcx>)>, ret: Ty<'tcx>) -> Self {
		Self { params, ret }
	}
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
				TyKind::Lazy => {
					unreachable!("Lazy type must be resolved before creating a union");
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
