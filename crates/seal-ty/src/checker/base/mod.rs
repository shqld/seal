mod decl;
mod expr;
mod narrow;
mod satisfies;
mod stmt;
mod ts_type;
mod widen;

use std::{cell::RefCell, collections::HashMap, fmt::Debug};

use crate::{
	Ty,
	context::{TyConstants, TyContext},
	sir::{Local, LocalId, Value},
	symbol::Symbol,
};
use swc_common::Span;

use super::errors::{Error, ErrorKind};

#[derive(Debug, Clone, Copy)]
struct Binding<'tcx> {
	ty: Ty<'tcx>,
	current: Option<Local<'tcx>>,
	is_assignable: bool,
}

pub struct BaseChecker<'tcx> {
	pub tcx: &'tcx TyContext<'tcx>,
	pub constants: TyConstants<'tcx>,
	bindings: RefCell<HashMap<Symbol, Binding<'tcx>>>,
	pub locals: RefCell<HashMap<LocalId, Value>>,
	pub errors: RefCell<Vec<Error<'tcx>>>,
}

impl Debug for BaseChecker<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("BaseChecker")
			.field("bindings", &self.bindings.borrow())
			.field("locals", &self.locals.borrow())
			.field("errors", &self.errors.borrow())
			.finish()
	}
}

impl<'tcx> BaseChecker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> BaseChecker<'tcx> {
		let constants = TyConstants::new(tcx);

		let checker = BaseChecker {
			tcx,
			constants,
			bindings: RefCell::new(HashMap::new()),
			locals: RefCell::new(HashMap::new()),
			errors: RefCell::new(vec![]),
		};

		// Register built-in types
		checker.set_binding(
			&Symbol::new((
				swc_atoms::Atom::new("Object"),
				swc_common::SyntaxContext::empty(),
			)),
			None,
			checker.constants.object,
			false,
		);
		checker.set_binding(
			&Symbol::new((
				swc_atoms::Atom::new("RegExp"),
				swc_common::SyntaxContext::empty(),
			)),
			None,
			checker.constants.regexp,
			false,
		);

		checker
	}

	pub fn add_local(&self, ty: Ty<'tcx>, value: Value) -> Local<'tcx> {
		let mut locals = self.locals.borrow_mut();
		let id = LocalId::new(locals.len());
		locals.insert(id, value);

		Local { ty, id }
	}

	pub fn new_scoped_checker(&self) -> BaseChecker<'tcx> {
		let checker = BaseChecker::new(self.tcx);
		let vars = self.bindings.borrow();

		for (name, binding) in vars.iter() {
			checker.set_binding(name, binding.current, binding.ty, binding.is_assignable);
		}

		checker
	}

	pub fn add_error_with_span(&self, err: ErrorKind<'tcx>, span: Span) {
		self.errors.borrow_mut().push(Error::new(err, span));
	}

	fn get_binding(&self, name: &Symbol) -> Option<Binding<'tcx>> {
		self.bindings.borrow().get(name).copied()
	}

	pub fn set_binding(
		&self,
		name: &Symbol,
		current: Option<Local<'tcx>>,
		ty: Ty<'tcx>,
		is_assignable: bool,
	) {
		self.bindings.borrow_mut().insert(name.clone(), Binding {
			ty,
			current,
			is_assignable,
		});
	}

	pub fn set_ty(&self, id: &Symbol, ty: Ty<'tcx>) {
		if let Some(var) = self.bindings.borrow_mut().get_mut(id) {
			var.ty = ty;
		} else {
			panic!("Variable not found: {:?}", id);
		}
	}

	// TODO: remove - use add_error_with_span directly
	pub fn raise_type_error(&self, expected: Ty<'tcx>, actual: Ty<'tcx>, span: Span) {
		self.add_error_with_span(ErrorKind::NotAssignable(expected, actual), span);
	}
}
