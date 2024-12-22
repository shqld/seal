use swc_atoms::Atom;
use swc_ecma_ast::{Id, TsType};

pub struct Module {
	pub functions: Vec<Function>,
}

pub struct Function {
	pub id: Id,
	pub params: Vec<Var>,
	pub body: Vec<Block>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockId(pub usize);

pub struct Block {
	pub id: BlockId,
	pub stmts: Vec<Stmt>,
	pub term: Option<Term>,
}

pub enum Term {
	Return,
	Goto(BlockId),
	Switch(Expr, BlockId, BlockId),
}

pub enum Stmt {
	Assign(Assign),
	Expr(Expr),
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Var {
	Id(Id),
	Ret,
}

pub struct Assign {
	pub var: Var,
	pub expr: Expr,
}

pub enum Expr {
	Var(Var),
	Const(Const),
}

pub enum Const {
	Number(f64),
	String(Atom),
	Boolean(bool),
}
