use swc_atoms::Atom;
use swc_common::SyntaxContext;
use swc_ecma_ast::Id;

use crate::Ty;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(Id);

impl Symbol {
	pub fn new(id: Id) -> Self {
		Self(id)
	}

	fn id(&self) -> &Id {
		&self.0
	}

	pub fn new_main() -> Self {
		Self((Atom::new("@main"), SyntaxContext::empty()))
	}

	pub fn new_ret() -> Self {
		Self((Atom::new("@ret"), SyntaxContext::empty()))
	}
}

impl std::fmt::Display for Symbol {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.id().0)
	}
}

#[derive(Debug, Clone)]
pub struct Module<'tcx> {
	pub functions: Vec<Function<'tcx>>,
}

#[derive(Debug, Clone)]
pub struct Function<'tcx> {
	pub name: Symbol,
	pub params: Vec<TypedVar<'tcx>>,
	pub ret: TypedVar<'tcx>,
	pub body: Vec<Block<'tcx>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypedVar<'tcx> {
	name: Symbol,
	ty: Ty<'tcx>,
}

impl<'tcx> TypedVar<'tcx> {
	pub fn new(name: Symbol, ty: Ty<'tcx>) -> Self {
		Self { name, ty }
	}

	pub fn name(&self) -> &Symbol {
		&self.name
	}

	pub fn ty(&self) -> Ty<'tcx> {
		self.ty
	}
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockId(usize);

impl BlockId {
	pub fn new(val: usize) -> Self {
		Self(val)
	}
}

#[derive(Debug, Clone)]
pub struct Block<'tcx> {
	id: BlockId,
	stmts: Vec<Stmt<'tcx>>,
	// TODO: no Option
	term: Option<Term>,
}

impl<'tcx> Block<'tcx> {
	pub fn new(id: BlockId) -> Self {
		Self {
			id,
			stmts: vec![],
			term: None,
		}
	}

	pub fn id(&self) -> BlockId {
		self.id
	}

	pub fn stmts(&self) -> &[Stmt<'tcx>] {
		&self.stmts
	}

	pub fn term(&self) -> Option<&Term> {
		self.term.as_ref()
	}

	pub fn add_stmt(&mut self, stmt: Stmt<'tcx>) {
		self.stmts.push(stmt);
	}

	pub fn set_term(&mut self, term: Term) {
		self.term = Some(term);
	}
}

#[derive(Debug, Clone)]
pub enum Term {
	Return,
	Goto(BlockId),
	Switch(Expr, BlockId, BlockId),
}

#[derive(Debug, Clone)]
pub enum Stmt<'tcx> {
	Let(Let<'tcx>),
	Assign(Assign),
	Ret(Option<Expr>),
	Expr(Expr),
	Satisfies(Expr, Ty<'tcx>),
}

#[derive(Debug, Clone)]
pub struct Let<'tcx> {
	var: TypedVar<'tcx>,
	init: Option<Expr>,
}

impl<'tcx> Let<'tcx> {
	pub fn new(var: TypedVar<'tcx>, init: Option<Expr>) -> Self {
		Self { var, init }
	}

	pub fn var(&self) -> &TypedVar<'tcx> {
		&self.var
	}

	pub fn init(&self) -> Option<&Expr> {
		self.init.as_ref()
	}
}

#[derive(Debug, Clone)]
pub struct Assign {
	left: Symbol,
	right: Expr,
}

impl Assign {
	pub fn new(left: Symbol, right: Expr) -> Self {
		Self { left, right }
	}

	pub fn left(&self) -> &Symbol {
		&self.left
	}

	pub fn right(&self) -> &Expr {
		&self.right
	}
}

#[derive(Debug, Clone)]
pub enum Expr {
	Var(Symbol),
	Const(Const),
}

#[derive(Debug, Clone)]
pub enum Const {
	Number(f64),
	String(Atom),
	Boolean(bool),
}
