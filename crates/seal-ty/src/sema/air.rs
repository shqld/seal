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
	Assign(Assign),
	Expr(Expr),
	Satisfies(Expr, Ty<'tcx>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Var {
	pub id: Id,
}

impl Var {
	pub fn new(id: Id) -> Self {
		Self { id }
	}

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
	var: Var,
	expr: Expr,
}

impl Assign {
	pub fn new(var: Var, expr: Expr) -> Self {
		Self { var, expr }
	}

	pub fn var(&self) -> &Var {
		&self.var
	}

	pub fn expr(&self) -> &Expr {
		&self.expr
	}
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
