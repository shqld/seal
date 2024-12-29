use swc_atoms::Atom;
use swc_ecma_ast::Id;

use crate::Ty;

#[derive(Debug, Clone)]
pub struct Module<'tcx> {
	pub functions: Vec<Function<'tcx>>,
}

#[derive(Debug, Clone)]
pub struct Function<'tcx> {
	pub id: Id,
	pub params: Vec<Param<'tcx>>,
	pub body: Vec<Block<'tcx>>,
	pub ret_ty: Ty<'tcx>,
}

#[derive(Debug, Clone)]
pub struct Param<'tcx> {
	// TODO: Var?
	pub id: Id,
	pub ty: Ty<'tcx>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockId(pub usize);

#[derive(Debug, Clone)]
pub struct Block<'tcx> {
	pub id: BlockId,
	pub stmts: Vec<Stmt<'tcx>>,
	pub term: Option<Term>,
}

#[derive(Debug, Clone)]
pub enum Term {
	Return,
	Goto(BlockId),
	Switch(Expr, BlockId, BlockId),
}

#[derive(Debug, Clone)]
pub enum Stmt<'tcx> {
	Assign(Assign),
	Expr(Expr),
	Satisfies(Expr, Ty<'tcx>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Var {
	Id(Id),
	Ret,
}

#[derive(Debug, Clone)]
pub struct Assign {
	pub var: Var,
	pub expr: Expr,
}

#[derive(Debug, Clone)]
pub enum Expr {
	Var(Var),
	Const(Const),
}

#[derive(Debug, Clone)]
pub enum Const {
	Number(f64),
	String(Atom),
	Boolean(bool),
}
