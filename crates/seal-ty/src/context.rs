use std::{
	cell::{Cell, RefCell},
	collections::{BTreeMap, BTreeSet, HashMap},
	fmt::Debug,
	rc::Rc,
};

use swc_atoms::Atom;
use swc_common::SyntaxContext;

use crate::{
	Ty, TyKind,
	intern::interner::Interner,
	kind::{Array, Class, Function, Interface, Object, Union},
	sir::{Def, DefId},
	symbol::Symbol,
};

pub struct TyContext<'tcx> {
	interner: Interner<'tcx, TyKind<'tcx>>,
	definitions: RefCell<HashMap<DefId, Def>>,
	definition_counter: Cell<usize>,
}

impl Debug for TyContext<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("TyContext")
			.field("definitions", &self.definitions.borrow())
			.finish()
	}
}

impl TyContext<'_> {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			interner: Interner::new(),
			definitions: RefCell::new(HashMap::new()),
			definition_counter: Cell::new(0),
		}
	}
}

impl TyContext<'_> {
	fn new_def_id(&self) -> DefId {
		let id = self.definition_counter.get();
		self.definition_counter.set(id + 1);

		DefId::new(id)
	}

	pub fn add_def(&self, def: Def) -> DefId {
		let id = self.new_def_id();
		let mut defs = self.definitions.borrow_mut();

		defs.insert(id, def);

		id
	}
}

impl<'tcx> TyContext<'tcx> {
	fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		Ty::new(self.interner.intern(kind))
	}

	pub fn new_const_string(&'tcx self, value: Atom) -> Ty<'tcx> {
		self.new_ty(TyKind::String(Some(value)))
	}

	pub fn new_function(&'tcx self, function: Function<'tcx>) -> Ty<'tcx> {
		self.new_ty(TyKind::Function(function))
	}

	pub fn new_class(&'tcx self, class: Class<'tcx>) -> Ty<'tcx> {
		self.new_ty(TyKind::Class(class))
	}

	pub fn new_interface(&'tcx self, interface: Rc<Interface<'tcx>>) -> Ty<'tcx> {
		self.new_ty(TyKind::Interface(interface))
	}

	pub fn new_union(&'tcx self, arms: BTreeSet<Ty<'tcx>>) -> Ty<'tcx> {
		match arms.len() {
			0 => self.new_ty(TyKind::Never),
			1 => *arms.first().unwrap(),
			_ => self.new_ty(TyKind::Union(Union::new(arms))),
		}
	}

	pub fn new_excluded_union(&'tcx self, uni: &Union<'tcx>, arm: Ty<'tcx>) -> Ty<'tcx> {
		let mut tys = uni.arms().clone();
		tys.remove(&arm);

		self.new_union(tys)
	}

	pub fn new_object(&'tcx self, obj: Object<'tcx>) -> Ty<'tcx> {
		self.new_ty(TyKind::Object(obj))
	}

	pub fn new_guard(&'tcx self, name: Symbol, ty: Ty<'tcx>) -> Ty<'tcx> {
		self.new_ty(TyKind::Guard(name, ty))
	}

	pub fn new_array(&'tcx self, element: Ty<'tcx>) -> Ty<'tcx> {
		self.new_ty(TyKind::Array(Array::new(element)))
	}
}

#[derive(Debug)]
pub struct TyConstants<'tcx> {
	pub boolean: Ty<'tcx>,
	pub number: Ty<'tcx>,
	pub string: Ty<'tcx>,
	pub err: Ty<'tcx>,
	pub void: Ty<'tcx>,
	pub never: Ty<'tcx>,
	pub lazy: Ty<'tcx>,
	pub unknown: Ty<'tcx>,
	pub null: Ty<'tcx>,
	pub object: Ty<'tcx>,
	pub regexp: Ty<'tcx>,

	pub type_of: Ty<'tcx>,

	pub proto_number: HashMap<Atom, Ty<'tcx>>,
	pub proto_string: HashMap<Atom, Ty<'tcx>>,
}

impl<'tcx> TyConstants<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> Self {
		let boolean = tcx.new_ty(TyKind::Boolean);
		let number = tcx.new_ty(TyKind::Number);
		let string = tcx.new_ty(TyKind::String(None));
		let err = tcx.new_ty(TyKind::Err);
		let void = tcx.new_ty(TyKind::Void);
		let never = tcx.new_ty(TyKind::Never);
		let lazy = tcx.new_ty(TyKind::Lazy);
		let unknown = tcx.new_ty(TyKind::Unknown);
		let null = tcx.new_ty(TyKind::Null);

		// Object type - represents the base Object type in JavaScript/TypeScript
		let object = tcx.new_interface(Rc::new(Interface::new(
			Symbol::new((Atom::new("Object"), SyntaxContext::empty())),
			BTreeMap::new(), // Base Object has no specific properties
		)));

		// RegExp is represented as an interface
		let regexp = tcx.new_interface(Rc::new(Interface::new(
			Symbol::new((Atom::new("RegExp"), SyntaxContext::empty())),
			[
				(Atom::new("source"), string),
				(Atom::new("global"), boolean),
				(Atom::new("ignoreCase"), boolean),
				(Atom::new("multiline"), boolean),
			]
			.into_iter()
			.collect(),
		)));

		macro_rules! parse_prop {
			($name:ident: $ty:expr) => {
				(Atom::new(stringify!($name)), $ty)
			};
		}

		macro_rules! parse_method {
            ($function_name:ident: ($($param_name:ident: $param_ty:expr),*) => $ret_ty:expr) => {
                (
                    Atom::new(stringify!($function_name)),
                    tcx.new_ty(TyKind::Function(Function::new(
                        vec![$((Symbol::new((Atom::new(stringify!($param_name)), SyntaxContext::empty())), $param_ty)),*],
                        $ret_ty,
                    )))
                )
            };
		}

		Self {
			boolean,
			number,
			string,
			err,
			void,
			never,
			lazy,
			unknown,
			null,
			object,
			regexp,

			type_of: tcx.new_union(
				["boolean", "number", "string"]
					.into_iter()
					.map(|s| tcx.new_const_string(Atom::new(s)))
					.collect(),
			),

			proto_number: [
				parse_method!(toExponential: (fractionDigits: number) => string),
				parse_method!(toFixed: (digits: number) => string),
				parse_method!(toLocaleString: () => string),
				parse_method!(toPrecision: (precision: number) => string),
			]
			.into_iter()
			.collect(),
			proto_string: [
				parse_prop!(length: number),
				parse_method!(at: (index: number) => string),
				parse_method!(charAt: (index: number) => string),
				parse_method!(charCodeAt: (index: number) => number),
				parse_method!(codePointAt: (index: number) => number),
				parse_method!(concat: (strings: string) => string),
				parse_method!(endsWith: (searchString: string) => boolean),
				parse_method!(includes: (searchString: string) => boolean),
				parse_method!(indexOf: (searchString: string) => number),
				parse_method!(isWellFormed: () => boolean),
				parse_method!(lastIndexOf: (searchString: string) => number),
				parse_method!(localeCompare: (compareString: string) => number),
				parse_method!(match: (regexp: string) => object),
				parse_method!(matchAll: (regexp: string) => object),
				parse_method!(normalize: (form: string) => string),
				parse_method!(padEnd: (targetLength: number, padString: string) => string),
				parse_method!(padStart: (targetLength: number, padString: string) => string),
				parse_method!(repeat: (count: number) => string),
				parse_method!(replace: (searchValue: string, replaceValue: string) => string),
				parse_method!(replaceAll: (searchValue: string, replaceValue: string) => string),
				parse_method!(search: (regexp: string) => number),
				parse_method!(slice: (start: number, end: number) => string),
				parse_method!(split: (separator: string, limit: number) => object),
				parse_method!(startsWith: (searchString: string, position: number) => boolean),
				parse_method!(substr: (start: number, length: number) => string),
				parse_method!(substring: (start: number, end: number) => string),
				parse_method!(toLocaleLowerCase: () => string),
				parse_method!(toLocaleUpperCase: () => string),
				parse_method!(toLowerCase: () => string),
				parse_method!(toUpperCase: () => string),
				parse_method!(toWellFormed: () => string),
				parse_method!(trim: () => string),
				parse_method!(trimEnd: () => string),
				parse_method!(trimLeft: () => string),
				parse_method!(trimRight: () => string),
				parse_method!(trimStart: () => string),
			]
			.into_iter()
			.collect(),
		}
	}
}
