pub mod air;
pub mod build;

use std::cell::{Cell, RefCell};

use swc_atoms::Atom;
use swc_common::SyntaxContext;

use air::{Block, BlockId, Function, Module, Stmt, Term, TypedVar};

use crate::{TyKind, context::TyContext, type_builder::TypeBuilder};

pub struct Sema<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
	ty_builder: TypeBuilder<'tcx>,
	global_block_counter: Cell<usize>,
	module: RefCell<Module<'tcx>>,
	function_stack: RefCell<Vec<Function<'tcx>>>,
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
			function_stack: RefCell::new(vec![Function {
				id: main_function_id.clone(),
				params: vec![],
				ret: air::TypedVar::new(
					air::Var::new_ret(&main_function_id),
					tcx.new_ty(TyKind::Void),
				),
				body: vec![Block {
					id: BlockId(0),
					stmts: vec![],
					term: None,
				}],
			}]),
		}
	}

	pub fn new_block(&self) -> Block<'tcx> {
		let val = self.global_block_counter.get() + 1;

		self.global_block_counter.set(val);

		Block {
			id: BlockId(val),
			stmts: vec![],
			term: None,
		}
	}

	pub fn add_block(&self, block: Block<'tcx>) {
		if let Some(function) = self.function_stack.borrow_mut().last_mut() {
			function.body.push(block);
		}
	}

	pub fn start_block(&self) {
		self.add_block(self.new_block());
	}

	pub fn add_stmt(&self, stmt: Stmt<'tcx>) {
		if let Some(function) = self.function_stack.borrow_mut().last_mut() {
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
		if let Some(function) = self.function_stack.borrow_mut().last_mut() {
			if let Some(block) = function.body.last_mut() {
				if block.term.is_none() {
					block.term = term;
				}
			}
		}
	}

	pub fn start_function(&self, function: Function<'tcx>) {
		self.function_stack.borrow_mut().push(function);
	}

	pub fn finish_function(&self) {
		self.finish_block(Some(Term::Return));

		if let Some(function) = self.function_stack.borrow_mut().pop() {
			self.module.borrow_mut().functions.push(function);
		} else {
			panic!("No function to finish");
		}
	}

	pub fn is_current_function_main(&self) -> bool {
		self.function_stack.borrow().len() == 1
	}

	pub fn get_current_function_ret(&self) -> TypedVar {
		self.function_stack.borrow().last().unwrap().ret.clone()
	}
}
