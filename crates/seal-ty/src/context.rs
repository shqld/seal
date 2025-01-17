use std::{
	collections::{BTreeMap, BTreeSet},
	rc::Rc,
};

use swc_atoms::Atom;

use crate::{
	Ty, TyKind,
	intern::interner::Interner,
	kind::{Class, Function, Interface, Object, Union},
	symbol::Symbol,
};

#[derive(Debug)]
pub struct TyContext<'tcx> {
	interner: Interner<'tcx, TyKind<'tcx>>,
}

impl TyContext<'_> {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			interner: Interner::new(),
		}
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

	pub fn new_object(&'tcx self, fields: BTreeMap<Atom, Ty<'tcx>>) -> Ty<'tcx> {
		self.new_ty(TyKind::Object(Object::new(fields)))
	}

	pub fn new_guard(&'tcx self, name: Symbol, ty: Ty<'tcx>) -> Ty<'tcx> {
		self.new_ty(TyKind::Guard(name, ty))
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

	pub type_of: Ty<'tcx>,
}

impl<'tcx> TyConstants<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> Self {
		Self {
			boolean: tcx.new_ty(TyKind::Boolean),
			number: tcx.new_ty(TyKind::Number),
			string: tcx.new_ty(TyKind::String(None)),
			err: tcx.new_ty(TyKind::Err),
			void: tcx.new_ty(TyKind::Void),
			never: tcx.new_ty(TyKind::Never),
			lazy: tcx.new_ty(TyKind::Lazy),

			type_of: tcx.new_union(
				["boolean", "number", "string"]
					.iter()
					.map(|s| tcx.new_const_string(Atom::new(*s)))
					.collect(),
			),
		}
	}
}
