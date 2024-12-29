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
	pub params: Vec<Var<'tcx>>,
	pub ret: Var<'tcx>,
	pub body: Vec<Block<'tcx>>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockId(pub usize);

#[derive(Debug, Clone)]
pub struct Block<'tcx> {
	pub id: BlockId,
	pub stmts: Vec<Stmt<'tcx>>,
	pub term: Option<Term<'tcx>>,
}

#[derive(Debug, Clone)]
pub enum Term<'tcx> {
	Return,
	Goto(BlockId),
	Switch(Expr<'tcx>, BlockId, BlockId),
}

#[derive(Debug, Clone)]
pub enum Stmt<'tcx> {
	Assign(Assign<'tcx>),
	Expr(Expr<'tcx>),
	Satisfies(Expr<'tcx>, Ty<'tcx>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Var<'tcx> {
	pub id: Id,
	// TODO: move to 'Assign'
	pub ty: Option<Ty<'tcx>>,
}

impl<'tcx> Var<'tcx> {
	pub fn new_ret(function_id: &Id, ty: Ty<'tcx>) -> Self {
		Self {
			id: (Atom::new("@ret"), function_id.1),
			ty: Some(ty),
		}
	}

	pub fn is_ret(&self) -> bool {
		self.id.0 == Atom::new("@ret")
	}
}

#[derive(Debug, Clone)]
pub struct Assign<'tcx> {
	pub var: Var<'tcx>,
	pub expr: Expr<'tcx>,
}

#[derive(Debug, Clone)]
pub enum Expr<'tcx> {
	Var(Var<'tcx>),
	Const(Const),
}

#[derive(Debug, Clone)]
pub enum Const {
	Number(f64),
	String(Atom),
	Boolean(bool),
}
