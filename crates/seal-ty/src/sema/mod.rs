pub mod air;
pub mod build;

use std::cell::{Cell, RefCell};

use swc_atoms::Atom;
use swc_common::SyntaxContext;
use swc_ecma_ast::Id;

use air::{Block, BlockId, Function, Module, Param, Stmt, Term};

use crate::{Ty, TyKind, context::TyContext, type_builder::TypeBuilder};

pub struct Sema<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
	ty_builder: TypeBuilder<'tcx>,
	global_block_counter: Cell<usize>,
	module: RefCell<Module<'tcx>>,
	functions: RefCell<Vec<Function<'tcx>>>,
}

impl<'tcx> Sema<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> Sema<'tcx> {
		let type_builder = TypeBuilder::new(tcx);
		let main_function_id = (Atom::new("@main"), SyntaxContext::empty());

		Sema {
			tcx,
			ty_builder: type_builder,
			global_block_counter: Cell::new(1),
			module: RefCell::new(Module { functions: vec![] }),
			functions: RefCell::new(vec![Function {
				id: main_function_id.clone(),
				params: vec![],
				body: vec![Block {
					id: BlockId(0),
					stmts: vec![],
					term: None,
				}],
				ret_ty: tcx.new_ty(TyKind::Void),
			}]),
		}
	}

	pub fn new_block(&self) -> Block {
		let val = self.global_block_counter.get() + 1;

		self.global_block_counter.set(val);

		Block {
			id: BlockId(val),
			stmts: vec![],
			term: None,
		}
	}

	pub fn add_block(&self, block: Block<'tcx>) {
		if let Some(function) = self.functions.borrow_mut().last_mut() {
			function.body.push(block);
		}
	}

	pub fn start_block(&'tcx self) {
		self.add_block(self.new_block());
	}

	pub fn add_stmt(&'tcx self, stmt: Stmt<'tcx>) {
		if let Some(function) = self.functions.borrow_mut().last_mut() {
			if let Some(block) = function.body.last_mut() {
				if block.term.is_some() {
					let mut block = self.new_block();
					block.stmts.push(stmt);
					function.body.push(block);
				} else {
					block.stmts.push(stmt);
				}
			}
		}
	}

	pub fn finish_block(&self, term: Option<Term>) {
		if let Some(function) = self.functions.borrow_mut().last_mut() {
			if let Some(block) = function.body.last_mut() {
				if block.term.is_none() {
					block.term = term;
				}
			}
		}
	}

	pub fn start_function(&'tcx self, id: Id, params: Vec<Param<'tcx>>, ret_ty: Ty<'tcx>) {
		let func = Function {
			id,
			params,
			body: vec![self.new_block()],
			ret_ty,
		};

		self.functions.borrow_mut().push(func);
	}

	pub fn finish_function(&self) {
		self.finish_block(Some(Term::Return));

		if let Some(function) = self.functions.borrow_mut().pop() {
			self.module.borrow_mut().functions.push(function);
		} else {
			panic!("No function to finish");
		}
	}

	pub fn is_current_function_main(&self) -> bool {
		self.functions.borrow().len() == 1
	}
}
