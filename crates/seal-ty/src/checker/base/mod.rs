mod expr;
mod narrow;
mod satisfies;
mod stmt;
mod ts_type;

use std::{cell::RefCell, collections::HashMap};

use crate::{
	Ty, TyKind,
	context::{TyConstants, TyContext},
	symbol::Symbol,
};

use super::scope::{ScopeStack, TyScope};

#[derive(Debug)]
struct Var<'tcx> {
	ty: Ty<'tcx>,
	is_assignable: bool,
	scoped_tys: HashMap<TyScope, Ty<'tcx>>,
}

#[derive(Debug)]
pub struct BaseChecker<'tcx> {
	pub tcx: &'tcx TyContext<'tcx>,
	pub constants: TyConstants<'tcx>,
	vars: RefCell<HashMap<Symbol, Var<'tcx>>>,
	errors: RefCell<Vec<String>>,
	scopes: RefCell<ScopeStack>,
}

impl<'tcx> BaseChecker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> BaseChecker<'tcx> {
		let constants = TyConstants::new(tcx);

		BaseChecker {
			tcx,
			vars: RefCell::new(HashMap::new()),
			constants,
			errors: RefCell::new(vec![]),
			scopes: RefCell::new(ScopeStack::new()),
		}
	}

	pub fn check(self) -> Result<(), Vec<String>> {
		let errors = self.errors.into_inner();

		if errors.is_empty() {
			Ok(())
		} else {
			Err(errors)
		}
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

	pub fn raise_type_error(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) {
		use TyKind::*;

		if matches!(actual.kind(), String(Some(_))) && !matches!(expected.kind(), String(_)) {
			self.add_error(format!(
				"expected '{expected}', got '{}'",
				TyKind::String(None)
			));
		} else {
			self.add_error(format!("expected '{expected}', got '{actual}'"));
		}
	}
}
