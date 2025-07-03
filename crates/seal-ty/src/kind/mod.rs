use std::{
	collections::{BTreeMap, BTreeSet},
	fmt::{Debug, Display},
	hash::Hash,
	ops::Deref,
	rc::Rc,
};

use swc_atoms::Atom;

use crate::{Ty, symbol::Symbol};

#[derive(Hash, PartialEq, Eq)]
pub enum TyKind<'tcx> {
	// value types
	Void,
	Boolean,
	Number,
	String(Option<Atom>),
	Object(Object<'tcx>),
	Function(Function<'tcx>),
	Class(Class<'tcx>),
	Interface(Rc<Interface<'tcx>>),
	Array(Array<'tcx>),
	Tuple(Tuple<'tcx>),

	// special types
	Union(Union<'tcx>),
	Generic(Generic<'tcx>),
	TypeParameter(TypeParameter),

	// internal checker types (users cannot create)
	Err,
	Lazy,
	Never,
	Unknown,
	Guard(Symbol, Ty<'tcx>),
}

impl Debug for TyKind<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(self, f)
	}
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
			TyKind::Class(Class { interface, .. }) => write!(f, "Class {}", interface.name),
			TyKind::Interface(interface) => write!(f, "{}", interface.name()),
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
			TyKind::Array(Array { element }) => write!(f, "{}[]", element),
			TyKind::Tuple(Tuple { elements }) => write!(
				f,
				"[{}]",
				elements
					.iter()
					.map(|ty| ty.to_string())
					.collect::<Vec<_>>()
					.join(", ")
			),
			TyKind::Generic(Generic { name, type_args }) => {
				if type_args.is_empty() {
					write!(f, "{}", name)
				} else {
					write!(
						f,
						"{}<{}>",
						name,
						type_args
							.iter()
							.map(|ty| ty.to_string())
							.collect::<Vec<_>>()
							.join(", ")
					)
				}
			}
			TyKind::TypeParameter(TypeParameter { name, .. }) => write!(f, "{}", name),
			TyKind::Err => write!(f, "<err>"),
			TyKind::Lazy => write!(f, "<lazy>",),
			TyKind::Never => write!(f, "<never>",),
			TyKind::Unknown => write!(f, "unknown"),
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
	pub fields: BTreeMap<Atom, Ty<'tcx>>,
}

impl<'tcx> Object<'tcx> {
	pub fn new(fields: BTreeMap<Atom, Ty<'tcx>>) -> Self {
		Self { fields }
	}

	pub fn get_prop(&self, key: &Atom) -> Option<Ty<'tcx>> {
		self.fields.get(key).copied()
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Class<'tcx> {
	ctor: Option<Function<'tcx>>,
	interface: Rc<Interface<'tcx>>,
}

impl<'tcx> Deref for Class<'tcx> {
	type Target = Interface<'tcx>;

	fn deref(&self) -> &Self::Target {
		&self.interface
	}
}

impl<'tcx> Class<'tcx> {
	pub fn new(ctor: Option<Function<'tcx>>, interface: Rc<Interface<'tcx>>) -> Self {
		Self {
			ctor,
			interface: interface.clone(),
		}
	}

	pub fn ctor(&self) -> Option<&Function<'tcx>> {
		self.ctor.as_ref()
	}

	pub fn interface(&self) -> Rc<Interface<'tcx>> {
		self.interface.clone()
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Interface<'tcx> {
	name: Symbol,
	fields: BTreeMap<Atom, Ty<'tcx>>,
}

impl<'tcx> Interface<'tcx> {
	pub fn new(name: Symbol, fields: BTreeMap<Atom, Ty<'tcx>>) -> Self {
		Self { name, fields }
	}

	pub fn name(&self) -> &Symbol {
		&self.name
	}

	pub fn fields(&self) -> &BTreeMap<Atom, Ty<'tcx>> {
		&self.fields
	}

	pub fn get_prop(&self, key: &Atom) -> Option<Ty<'tcx>> {
		self.fields.get(key).copied()
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Array<'tcx> {
	pub element: Ty<'tcx>,
}

impl<'tcx> Array<'tcx> {
	pub fn new(element: Ty<'tcx>) -> Self {
		Self { element }
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Tuple<'tcx> {
	pub elements: Vec<Ty<'tcx>>,
}

impl<'tcx> Tuple<'tcx> {
	pub fn new(elements: Vec<Ty<'tcx>>) -> Self {
		Self { elements }
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Generic<'tcx> {
	pub name: Symbol,
	pub type_args: Vec<Ty<'tcx>>,
}

impl<'tcx> Generic<'tcx> {
	pub fn new(name: Symbol, type_args: Vec<Ty<'tcx>>) -> Self {
		Self { name, type_args }
	}
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct TypeParameter {
	pub name: Symbol,
	pub constraint: Option<Box<TyKind<'static>>>,
	pub default: Option<Box<TyKind<'static>>>,
}

impl TypeParameter {
	pub fn new(
		name: Symbol,
		constraint: Option<Box<TyKind<'static>>>,
		default: Option<Box<TyKind<'static>>>,
	) -> Self {
		Self {
			name,
			constraint,
			default,
		}
	}
}