pub mod check;
pub mod parse;
mod satisfies;

use std::cell::Cell;

use swc_atoms::Atom;
use swc_common::SyntaxContext;

use crate::{Ty, TyKind, context::TyContext, sema::air::Function, type_builder::TypeBuilder};

pub struct TypeChecker<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
	_ty_builder: TypeBuilder<'tcx>,
	syntax_context: Cell<SyntaxContext>,
}

impl<'tcx> TypeChecker<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> TypeChecker<'tcx> {
		let _ty_builder = TypeBuilder::new(tcx);
		TypeChecker {
			tcx,
			_ty_builder,
			syntax_context: Cell::new(SyntaxContext::empty()),
		}
	}

	pub fn new_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
		self.tcx.new_ty(kind)
	}

	pub fn enter_function(&self, function: &Function<'tcx>) {
		self.syntax_context.replace(function.id.1);
	}

	pub fn get_ret_ty(&self) -> Ty<'tcx> {
		self.tcx
			.get_ty(&(Atom::new("@ret"), self.syntax_context.get()))
			.expect("Return type not found")
	}

	pub fn set_ret_ty(&self, ty: Ty<'tcx>) {
		self.tcx
			.set_ty((Atom::new("@ret"), self.syntax_context.get()), ty);
	}
}
