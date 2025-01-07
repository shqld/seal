pub mod check;
mod narrow;
pub mod parse;
mod satisfies;

use std::{
	cell::{Cell, RefCell},
	collections::HashMap,
};

use crate::{
	Ty,
	context::{BlockId, TyConstants, TyContext},
	symbol::Symbol,
};

struct VarInfo {
	can_be_assigned: bool,
}

pub struct Function<'tcx> {
	name: Symbol,
	ret: Ty<'tcx>,
	vars: HashMap<Symbol, VarInfo>,
	block_ids: Vec<BlockId>,
	has_returned: bool,
}

pub struct Checker<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
	types: RefCell<HashMap<Symbol, Ty<'tcx>>>,
	type_overrides: RefCell<HashMap<(Symbol, BlockId), Ty<'tcx>>>,
	constants: TyConstants<'tcx>,
	errors: RefCell<Vec<String>>,
	functions: RefCell<Vec<Function<'tcx>>>,
	block_id_counter: Cell<usize>,
}

impl<'tcx> Checker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> Checker<'tcx> {
		let constants = TyConstants::new(tcx);

		Checker {
			tcx,
			types: RefCell::new(HashMap::new()),
			type_overrides: RefCell::new(HashMap::new()),
			constants,
			errors: RefCell::new(vec![]),
			functions: RefCell::new(vec![]),
			block_id_counter: Cell::new(0),
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

	pub fn add_error(&self, error: String) {
		self.errors.borrow_mut().push(error);
	}

	fn new_block_id(&self) -> BlockId {
		let id = self.block_id_counter.get();
		self.block_id_counter.set(id + 1);
		BlockId(id)
	}

	pub fn start_function(&self, name: &Symbol, params: Vec<Symbol>, ret: Ty<'tcx>) {
		self.functions.borrow_mut().push(Function {
			name: name.clone(),
			ret,
			vars: params
				.into_iter()
				.map(|param| {
					(param, VarInfo {
						can_be_assigned: false,
					})
				})
				.collect(),
			block_ids: vec![self.new_block_id()],
			has_returned: false,
		});
	}

	pub fn finish_function(&self) -> Function<'tcx> {
		self.functions.borrow_mut().pop().unwrap()
	}

	pub fn get_current_function_ret(&self) -> Ty<'tcx> {
		self.functions.borrow().last().unwrap().ret
	}

	pub fn get_current_function_has_returned(&self) -> bool {
		self.functions.borrow().last().unwrap().has_returned
	}

	pub fn set_current_function_has_returned(&self, has_returned: bool) {
		self.functions.borrow_mut().last_mut().unwrap().has_returned = has_returned;
	}

	pub fn get_current_block_id(&self) -> BlockId {
		*self
			.functions
			.borrow()
			.last()
			.unwrap()
			.block_ids
			.last()
			.unwrap()
	}

	pub fn push_current_block_id(&self) {
		self.functions
			.borrow_mut()
			.last_mut()
			.unwrap()
			.block_ids
			.push(self.new_block_id());
	}

	pub fn pop_current_block_id(&self) {
		self.functions
			.borrow_mut()
			.last_mut()
			.unwrap()
			.block_ids
			.pop();
	}

	pub fn add_var_entry(&self, symbol: &Symbol, can_be_assigned: bool) {
		self.functions
			.borrow_mut()
			.last_mut()
			.unwrap()
			.vars
			.insert(symbol.clone(), VarInfo { can_be_assigned });
	}

	pub fn is_var_can_be_assigned(&self, symbol: &Symbol) -> bool {
		self.functions
			.borrow()
			.last()
			.unwrap()
			.vars
			.get(symbol)
			.unwrap()
			.can_be_assigned
	}
}
