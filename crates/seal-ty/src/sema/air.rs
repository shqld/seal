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
	pub params: Vec<TypedVar<'tcx>>,
	pub ret: TypedVar<'tcx>,
	pub body: Vec<Block<'tcx>>,
	// TODO: decls
	// pub decls: Vec<TypedVar<'tcx>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypedVar<'tcx>(Var, Ty<'tcx>);

impl<'tcx> TypedVar<'tcx> {
	pub fn new(var: Var, ty: Ty<'tcx>) -> Self {
		Self(var, ty)
	}

	pub fn is_ret(&self) -> bool {
		self.0.is_ret()
	}

	pub fn var(&self) -> &Var {
		&self.0
	}

	pub fn ty(&self) -> Ty<'tcx> {
		self.1
	}
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
pub struct Var {
	pub id: Id,
}

impl Var {
	// TODO: new

	pub fn new_ret(function_id: &Id) -> Self {
		Self {
			id: (Atom::new("@ret"), function_id.1),
		}
	}

	pub fn is_ret(&self) -> bool {
		self.id.0 == Atom::new("@ret")
	}
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
