pub mod air;
pub mod build;

use std::cell::{Cell, RefCell};

use air::{Assign, Block, BlockId, Expr, Function, Module, Stmt, Symbol, Term, TypedVar, Var};

use crate::{Ty, TyKind, context::TyContext, type_builder::TypeBuilder};

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
		let main_function_name = Symbol::new_main();

		Sema {
			tcx,
			ty_builder: type_builder,
			global_block_counter: Cell::new(1),
			module: RefCell::new(Module { functions: vec![] }),
			function_stack: RefCell::new(vec![Function {
				name: main_function_name.clone(),
				params: vec![],
				ret: TypedVar::new(
					Var::new(Symbol::new_ret(&main_function_name)),
					tcx.new_ty(TyKind::Void),
				),
				body: vec![Block::new(BlockId::new(0))],
			}]),
		}
	}

	pub fn new_block(&self) -> Block<'tcx> {
		let val = self.global_block_counter.get() + 1;

		self.global_block_counter.set(val);

		Block::new(BlockId::new(val))
	}

	pub fn add_block(&self, block: Block<'tcx>) {
		if let Some(function) = self.function_stack.borrow_mut().last_mut() {
			if let Some(current_block) = function.body.last_mut() {
				if current_block.term().is_none() {
					current_block.set_term(Term::Goto(block.id()));
				}
			}
			function.body.push(block);
		} else {
			panic!("No function to add block to");
		}
	}

	pub fn start_block(&self) {
		self.add_block(self.new_block());
	}

	pub fn finish_block(&self, term: Term) {
		if let Some(function) = self.function_stack.borrow_mut().last_mut() {
			if let Some(block) = function.body.last_mut() {
				if block.term().is_some() {
					panic!("Block already terminated");
				} else {
					block.set_term(term);
				}
			}
		}
	}

	pub fn start_function(&self, function: Function<'tcx>) {
		self.function_stack.borrow_mut().push(function);
	}

	pub fn finish_function(&self) {
		if let Some(mut function) = self.function_stack.borrow_mut().pop() {
			if let Some(block) = function.body.last_mut() {
				if let Some(term) = block.term() {
					if !matches!(term, Term::Return) {
						panic!("Function does not return");
					}
				} else {
					block.set_term(Term::Return);
				}
			}
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

	pub fn add_stmt(&self, stmt: Stmt<'tcx>) {
		if let Some(function) = self.function_stack.borrow_mut().last_mut() {
			if let Some(block) = function.body.last_mut() {
				block.add_stmt(stmt);
			}
		}
	}

	pub fn add_assign_stmt(&self, var: Var, expr: Expr) {
		self.add_stmt(Stmt::Assign(Assign::new(var, expr)));
	}

	pub fn add_expr_stmt(&self, expr: Expr) {
		self.add_stmt(Stmt::Expr(expr));
	}

	pub fn add_satisfies_stmt(&self, expr: Expr, ty: Ty<'tcx>) {
		self.add_stmt(Stmt::Satisfies(expr, ty));
	}
}
