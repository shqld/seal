use std::{
	cell::RefCell,
	collections::{BTreeMap, BTreeSet, HashMap},
};

use swc_atoms::Atom;

use crate::{
	Ty, TyKind,
	builder::sir::{BlockId, Symbol},
	infer::InferContext,
	interner::interner::Interner,
	kind::{Function, Object, Union},
};

#[derive(Debug)]
pub struct TyContext<'tcx> {
	interner: Interner<'tcx, TyKind<'tcx>>,
	pub infer: InferContext<'tcx>,
	types: RefCell<HashMap<Symbol, Ty<'tcx>>>,
	type_overrides: RefCell<HashMap<(Symbol, BlockId), Ty<'tcx>>>,
}

impl<'tcx> TyContext<'tcx> {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self {
			interner: Interner::new(),
			infer: InferContext::new(),
			types: RefCell::new(HashMap::new()),
			type_overrides: RefCell::new(HashMap::new()),
		}
	}

	pub fn get_ty(&self, id: &Symbol, block_id: BlockId) -> Option<Ty<'tcx>> {
		self.type_overrides
			.borrow()
			.get(&(id.clone(), block_id))
			.cloned()
			.or_else(|| self.types.borrow().get(id).cloned())
	}

	pub fn set_ty(&self, id: &Symbol, ty: Ty<'tcx>) {
		self.types.borrow_mut().insert(id.clone(), ty);
	}

	pub fn override_ty(&self, id: &Symbol, block_id: BlockId, ty: Ty<'tcx>) {
		self.type_overrides
			.borrow_mut()
			.insert((id.clone(), block_id), ty);
	}
}

impl<'tcx> TyContext<'tcx> {
	fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		Ty::new(self.interner.intern(kind))
	}

	pub fn new_const_string(&'tcx self, value: Atom) -> Ty<'tcx> {
		self.new_ty(TyKind::String(Some(value)))
	}

	pub fn new_infer_ty(&'tcx self) -> Ty<'tcx> {
		let id = self.infer.new_id();
		Ty::new(self.interner.intern(TyKind::Infer(id)))
	}

	pub fn new_function(&'tcx self, params: Vec<Ty<'tcx>>, ret: Ty<'tcx>) -> Ty<'tcx> {
		self.new_ty(TyKind::Function(Function { params, ret }))
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

	pub type_of: Ty<'tcx>,
}

impl<'tcx> TyConstants<'tcx> {
	pub fn new(factory: &'tcx TyContext<'tcx>) -> Self {
		Self {
			boolean: factory.new_ty(TyKind::Boolean),
			number: factory.new_ty(TyKind::Number),
			string: factory.new_ty(TyKind::String(None)),
			err: factory.new_ty(TyKind::Err),
			void: factory.new_ty(TyKind::Void),
			never: factory.new_ty(TyKind::Never),

			type_of: factory.new_union(
				["boolean", "number", "string"]
					.iter()
					.map(|s| factory.new_const_string(Atom::new(*s)))
					.collect(),
			),
		}
	}
}
