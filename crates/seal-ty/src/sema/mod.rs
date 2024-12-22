pub mod air;
pub mod build;

use std::cell::{Cell, RefCell};

use swc_atoms::Atom;
use swc_common::SyntaxContext;
use swc_ecma_ast::Id;

use air::{Block, BlockId, Function, Stmt, Var};

pub struct Sema {
	ctx: Context,
	block_counter: Cell<usize>,
	pub block: RefCell<Block>,
	pub functions: Vec<Function>,
}

pub struct Context {}

impl Context {
	pub fn new() -> Context {
		Context {}
	}
}

impl Sema {
	pub fn new() -> Sema {
		Sema {
			ctx: Context::new(),
			block_counter: Cell::new(1),
			block: RefCell::new(Block {
				id: BlockId(0),
				stmts: vec![],
				term: None,
			}),
			functions: vec![Function {
				id: (Atom::new("@main"), SyntaxContext::empty()),
				params: vec![],
				body: vec![],
			}],
		}
	}

	pub fn new_block(&self) -> Block {
		let val = self.block_counter.get() + 1;

		self.block_counter.set(val);

		Block {
			id: BlockId(val),
			stmts: vec![],
			term: None,
		}
	}

	pub fn add_block(&self, block: Block) {
		*self.block.borrow_mut() = block;
	}

	pub fn push_block(&self) {
		self.add_block(self.new_block());
	}

	pub fn add_stmt(&self, stmt: Stmt) {
		let mut block = self.block.borrow_mut();

		if block.term.is_some() {
			self.push_block();
		}

		block.stmts.push(stmt);
	}

	pub fn push_function(&self, id: Id, params: Vec<Var>) -> Function {
		Function {
			id,
			params,
			body: vec![self.new_block()],
		}
	}
}
