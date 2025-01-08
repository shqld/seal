pub mod check;
mod narrow;
mod satisfies;
mod scope;

use std::{cell::RefCell, collections::HashMap};

use scope::{ScopeStack, TyScope};

use crate::{
	Ty,
	context::{TyConstants, TyContext},
	symbol::Symbol,
};

#[derive(Debug)]
struct Function<'tcx> {
	name: Symbol,
	ret: Ty<'tcx>,
	params: Vec<Ty<'tcx>>,
	has_returned: bool,
	root_scope: TyScope,
}

#[derive(Debug)]
struct Var<'tcx> {
	ty: Ty<'tcx>,
	is_assignable: bool,
	scoped_tys: HashMap<TyScope, Ty<'tcx>>,
}

#[derive(Debug)]
pub struct Checker<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
	vars: RefCell<HashMap<Symbol, Var<'tcx>>>,
	constants: TyConstants<'tcx>,
	errors: RefCell<Vec<String>>,
	functions: RefCell<Vec<Function<'tcx>>>,
	scopes: RefCell<ScopeStack>,
}

impl<'tcx> Checker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> Checker<'tcx> {
		let constants = TyConstants::new(tcx);

		Checker {
			tcx,
			vars: RefCell::new(HashMap::new()),
			constants,
			errors: RefCell::new(vec![]),
			functions: RefCell::new(vec![]),
			scopes: RefCell::new(ScopeStack::new()),
		}
	}

	pub fn get_var_ty(&self, id: &Symbol) -> Option<Ty<'tcx>> {
		let scope = self.get_current_scope();

		self.vars
			.borrow()
			.get(id)
			.map(|var| var.scoped_tys.get(&scope).copied().unwrap_or(var.ty))
	}

	pub fn get_var_is_assignable(&self, id: &Symbol) -> Option<bool> {
		self.vars.borrow().get(id).map(|var| var.is_assignable)
	}

	pub fn add_var(&self, id: &Symbol, ty: Ty<'tcx>, is_assignable: bool) {
		self.vars.borrow_mut().insert(id.clone(), Var {
			ty,
			is_assignable,
			scoped_tys: HashMap::new(),
		});
	}

	pub fn add_scoped_ty(&self, id: &Symbol, scope: TyScope, ty: Ty<'tcx>) {
		let mut vars = self.vars.borrow_mut();

		if let Some(var) = vars.get_mut(id) {
			var.scoped_tys.insert(scope, ty);
		}
	}

	pub fn add_error(&self, error: String) {
		self.errors.borrow_mut().push(error);
	}

	pub fn start_function(&self, name: &Symbol, params: Vec<(Symbol, Ty<'tcx>)>, ret: Ty<'tcx>) {
		let mut param_tys = vec![];

		for (name, ty) in &params {
			let ty = *ty;

			param_tys.push(ty);
			self.add_var(name, ty, false);
		}

		let root_scope = self.enter_new_scope();

		self.functions.borrow_mut().push(Function {
			name: name.clone(),
			ret,
			params: param_tys,
			has_returned: false,
			root_scope,
		});
	}

	pub fn finish_function(&self) {
		let function = self.functions.borrow_mut().pop().unwrap();

		assert_eq!(function.root_scope, self.get_current_scope());

		self.leave_current_scope();

		self.add_var(
			&function.name,
			self.tcx.new_function(function.params, function.ret),
			false,
		);
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

	pub fn get_current_scope(&self) -> TyScope {
		self.scopes.borrow().peek()
	}

	pub fn enter_new_scope(&self) -> TyScope {
		self.scopes.borrow_mut().push()
	}

	pub fn leave_current_scope(&self) {
		self.scopes.borrow_mut().pop();
	}
}
