pub mod air;
pub mod build;

use std::{
	cell::{Cell, RefCell},
	collections::HashMap,
};

use air::{Assign, Block, BlockId, Expr, Function, Let, Module, Stmt, Symbol, Term, TypedVar};

use crate::{Ty, TyKind, context::TyContext};

pub struct Sema<'tcx> {
	tcx: &'tcx TyContext<'tcx>,
	global_block_counter: Cell<usize>,
	module: RefCell<Module<'tcx>>,
	function_stack: RefCell<Vec<Function<'tcx>>>,
	var_table: RefCell<HashMap<Symbol, VarInfo>>,
}

pub struct VarInfo {
	can_be_assigned: bool,
}

impl<'tcx> Sema<'tcx> {
	pub fn new(tcx: &'tcx TyContext<'tcx>) -> Sema<'tcx> {
		Sema {
			tcx,
			global_block_counter: Cell::new(1),
			module: RefCell::new(Module { functions: vec![] }),
			function_stack: RefCell::new(vec![Function {
				name: Symbol::new_main(),
				params: vec![],
				ret: TypedVar::new(Symbol::new_ret(), tcx.new_ty(TyKind::Void)),
				body: vec![Block::new(BlockId::new(0))],
			}]),
			var_table: RefCell::new(HashMap::new()),
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

	pub fn start_function(&self, name: &Symbol, params: Vec<TypedVar<'tcx>>, ret: TypedVar<'tcx>) {
		self.global_block_counter.set(0);

		self.function_stack.borrow_mut().push(Function {
			name: name.clone(),
			params,
			ret,
			body: vec![self.new_block()],
		});
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

	pub fn add_let_stmt(&self, name: Symbol, ty: Ty<'tcx>, init: Option<Expr>) {
		self.add_stmt(Stmt::Let(Let::new(TypedVar::new(name, ty), init)));
	}

	pub fn add_assign_stmt(&self, name: Symbol, expr: Expr) {
		self.add_stmt(Stmt::Assign(Assign::new(name, expr)));
	}

	pub fn add_ret_stmt(&self, expr: Option<Expr>) {
		self.add_stmt(Stmt::Ret(expr));
	}

	pub fn add_expr_stmt(&self, expr: Expr) {
		self.add_stmt(Stmt::Expr(expr));
	}

	pub fn add_satisfies_stmt(&self, expr: Expr, ty: Ty<'tcx>) {
		self.add_stmt(Stmt::Satisfies(expr, ty));
	}

	pub fn add_var_entry(&self, symbol: Symbol, can_be_assigned: bool) {
		self.var_table
			.borrow_mut()
			.insert(symbol, VarInfo { can_be_assigned });
	}

	pub fn is_var_can_be_assigned(&self, symbol: &Symbol) -> bool {
		self.var_table.borrow().get(symbol).unwrap().can_be_assigned
	}
}
