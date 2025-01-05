use std::{
	cell::RefCell,
	collections::{BTreeSet, HashMap},
};

use swc_atoms::Atom;

use crate::{
	Ty, TyKind,
	builder::sir::{BlockId, Symbol},
	infer::InferContext,
	interner::interner::Interner,
	kind::{Function, Union},
};

#[derive(Debug)]
pub struct TyContext<'tcx> {
	pub interner: Interner<'tcx, TyKind<'tcx>>,
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

	pub fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		Ty::new(self.interner.intern(kind))
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
	pub fn new_boolean(&'tcx self) -> Ty<'tcx> {
		self.new_ty(TyKind::Boolean)
	}

	pub fn new_number(&'tcx self) -> Ty<'tcx> {
		self.new_ty(TyKind::Number)
	}

	pub fn new_string(&'tcx self) -> Ty<'tcx> {
		self.new_ty(TyKind::String(None))
	}

	pub fn new_const_string(&'tcx self, value: Atom) -> Ty<'tcx> {
		self.new_ty(TyKind::String(Some(value)))
	}

	pub fn new_void(&'tcx self) -> Ty<'tcx> {
		self.new_ty(TyKind::Void)
	}

	pub fn new_infer_ty(&'tcx self) -> Ty<'tcx> {
		let id = self.infer.new_id();
		Ty::new(self.interner.intern(TyKind::Infer(id)))
	}

	pub fn new_function(&'tcx self, params: Vec<Ty<'tcx>>, ret: Ty<'tcx>) -> Ty<'tcx> {
		self.new_ty(TyKind::Function(Function { params, ret }))
	}

	pub fn new_union(&'tcx self, tys: BTreeSet<Ty<'tcx>>) -> Ty<'tcx> {
		match tys.len() {
			0 => self.new_ty(TyKind::Never),
			1 => *tys.first().unwrap(),
			_ => self.new_ty(TyKind::Union(Union::new(tys))),
		}
	}

	pub fn new_excluded_union(&'tcx self, uni: &Union<'tcx>, ty: Ty<'tcx>) -> Ty<'tcx> {
		let mut tys = uni.tys().clone();
		tys.remove(&ty);

		self.new_union(tys)
	}
}
