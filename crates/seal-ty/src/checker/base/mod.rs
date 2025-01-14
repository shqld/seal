mod decl;
mod expr;
mod narrow;
mod satisfies;
mod stmt;
mod ts_type;

use std::{cell::RefCell, collections::HashMap, ops::Deref};

use crate::{
	Ty,
	context::{TyConstants, TyContext},
	symbol::Symbol,
};

use super::errors::{Error, ErrorKind};

#[derive(Debug)]
struct Var<'tcx> {
	ty: Ty<'tcx>,
	is_assignable: bool,
}

#[derive(Debug)]
pub struct BaseChecker<'tcx> {
	pub tcx: &'tcx TyContext<'tcx>,
	pub constants: TyConstants<'tcx>,
	vars: RefCell<HashMap<Symbol, Var<'tcx>>>,
	pub errors: RefCell<Vec<Error<'tcx>>>,
}

impl<'tcx> BaseChecker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> BaseChecker<'tcx> {
		let constants = TyConstants::new(tcx);

		BaseChecker {
			tcx,
			vars: RefCell::new(HashMap::new()),
			constants,
			errors: RefCell::new(vec![]),
		}
	}

	pub fn new_scoped_checker(&self) -> BaseChecker<'tcx> {
		let checker = BaseChecker::new(self.tcx);

		for (id, var) in self.vars.borrow().deref() {
			checker.add_var(id, var.ty, var.is_assignable);
		}

		checker
	}

	pub fn add_error(&self, err: ErrorKind<'tcx>) {
		self.errors.borrow_mut().push(Error::new(err));
	}

	pub fn get_var_ty(&self, id: &Symbol) -> Option<Ty<'tcx>> {
		self.vars.borrow().get(id).map(|var| var.ty)
	}

	pub fn get_var_is_assignable(&self, id: &Symbol) -> Option<bool> {
		self.vars.borrow().get(id).map(|var| var.is_assignable)
	}

	pub fn add_var(&self, id: &Symbol, ty: Ty<'tcx>, is_assignable: bool) {
		self.vars
			.borrow_mut()
			.insert(id.clone(), Var { ty, is_assignable });
	}

	pub fn set_var(&self, id: &Symbol, ty: Ty<'tcx>) {
		if let Some(var) = self.vars.borrow_mut().get_mut(id) {
			var.ty = ty;
		} else {
			panic!("Variable not found: {:?}", id);
		}
	}

	// TODO: remove
	pub fn raise_type_error(&self, expected: Ty<'tcx>, actual: Ty<'tcx>) {
		self.add_error(ErrorKind::NotAssignable(expected, actual));
	}
}
