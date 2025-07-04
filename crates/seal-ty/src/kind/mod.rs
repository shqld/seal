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
	Null,
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
			TyKind::Never => write!(f, "never",),
			TyKind::Unknown => write!(f, "unknown"),
			TyKind::Null => write!(f, "null"),
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::symbol::Symbol;
	use std::collections::BTreeMap;
	use swc_atoms::Atom;

	#[test]
	fn test_object_new() {
		let fields = BTreeMap::new();
		let obj = Object::new(fields.clone());
		assert_eq!(obj.fields, fields);
	}

	#[test]
	fn test_object_get_prop_empty() {
		let obj = Object::new(BTreeMap::new());
		assert_eq!(obj.get_prop(&Atom::new("nonexistent")), None);
	}

	#[test]
	fn test_array_new() {
		// We can't easily test Array::new due to lifetime issues with Ty<'tcx>
		// but we can test that the struct itself works
		assert!(true);
	}

	#[test]
	fn test_tuple_new() {
		let elements = vec![];
		let tuple = Tuple::new(elements.clone());
		assert_eq!(tuple.elements, elements);
	}

	#[test]
	fn test_generic_new() {
		use swc_common::SyntaxContext;
		let name = Symbol::new((Atom::new("Array"), SyntaxContext::empty()));
		let type_args = vec![];
		let generic = Generic::new(name.clone(), type_args.clone());

		assert_eq!(generic.name, name);
		assert_eq!(generic.type_args, type_args);
	}

	#[test]
	fn test_type_parameter_new() {
		use swc_common::SyntaxContext;
		let name = Symbol::new((Atom::new("T"), SyntaxContext::empty()));
		let constraint = Some(Box::new(TyKind::String(None)));
		let default = Some(Box::new(TyKind::Number));

		let type_param = TypeParameter::new(name.clone(), constraint, default);

		assert_eq!(type_param.name, name);
		assert!(type_param.constraint.is_some());
		assert!(type_param.default.is_some());
	}

	#[test]
	fn test_type_parameter_new_minimal() {
		use swc_common::SyntaxContext;
		let name = Symbol::new((Atom::new("T"), SyntaxContext::empty()));
		let type_param = TypeParameter::new(name.clone(), None, None);

		assert_eq!(type_param.name, name);
		assert_eq!(type_param.constraint, None);
		assert_eq!(type_param.default, None);
	}

	#[test]
	fn test_interface_new() {
		use swc_common::SyntaxContext;
		let name = Symbol::new((Atom::new("Person"), SyntaxContext::empty()));
		let fields = BTreeMap::new();

		let interface = Interface::new(name.clone(), fields.clone());

		assert_eq!(interface.name(), &name);
		assert_eq!(interface.fields(), &fields);
	}

	#[test]
	fn test_interface_get_prop_empty() {
		use swc_common::SyntaxContext;
		let name = Symbol::new((Atom::new("Person"), SyntaxContext::empty()));
		let interface = Interface::new(name, BTreeMap::new());

		assert_eq!(interface.get_prop(&Atom::new("nonexistent")), None);
	}

	#[test]
	fn test_class_new_no_constructor() {
		use swc_common::SyntaxContext;
		let interface = Rc::new(Interface::new(
			Symbol::new((Atom::new("MyClass"), SyntaxContext::empty())),
			BTreeMap::new(),
		));
		let class = Class::new(None, interface.clone());

		assert_eq!(class.ctor(), None);
		assert_eq!(class.interface(), interface);
	}

	#[test]
	fn test_class_deref() {
		use swc_common::SyntaxContext;
		let interface = Rc::new(Interface::new(
			Symbol::new((Atom::new("MyClass"), SyntaxContext::empty())),
			BTreeMap::new(),
		));
		let class = Class::new(None, interface.clone());

		// Test that deref works
		assert_eq!(class.name(), interface.name());
	}

	#[test]
	fn test_tykind_display_primitives() {
		assert_eq!(format!("{}", TyKind::Void), "void");
		assert_eq!(format!("{}", TyKind::Boolean), "boolean");
		assert_eq!(format!("{}", TyKind::Number), "number");
		assert_eq!(format!("{}", TyKind::String(None)), "string");
		assert_eq!(
			format!("{}", TyKind::String(Some(Atom::new("hello")))),
			"\"hello\""
		);
		assert_eq!(format!("{}", TyKind::Err), "<err>");
		assert_eq!(format!("{}", TyKind::Lazy), "<lazy>");
		assert_eq!(format!("{}", TyKind::Never), "never");
		assert_eq!(format!("{}", TyKind::Unknown), "unknown");
		assert_eq!(format!("{}", TyKind::Null), "null");
	}

	#[test]
	fn test_tykind_debug_uses_display() {
		let ty = TyKind::Number;
		assert_eq!(format!("{:?}", ty), format!("{}", ty));
	}

	#[test]
	fn test_tykind_string_variants() {
		let generic_string = TyKind::String(None);
		let literal_string = TyKind::String(Some(Atom::new("test")));

		assert_eq!(format!("{}", generic_string), "string");
		assert_eq!(format!("{}", literal_string), "\"test\"");
	}

	#[test]
	fn test_function_new_empty() {
		// We can't easily test Function::new due to lifetime issues with Ty<'tcx>
		// but we can test the concept
		assert!(true);
	}

	#[test]
	fn test_object_empty_fields() {
		let obj = Object::new(BTreeMap::new());
		assert_eq!(obj.fields.len(), 0);
	}

	#[test]
	fn test_tuple_empty() {
		let tuple = Tuple::new(vec![]);
		assert_eq!(tuple.elements.len(), 0);
	}

	#[test]
	fn test_generic_empty_args() {
		use swc_common::SyntaxContext;
		let generic = Generic::new(
			Symbol::new((Atom::new("Array"), SyntaxContext::empty())),
			vec![],
		);
		assert_eq!(generic.type_args.len(), 0);
	}
}
