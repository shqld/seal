use std::{collections::HashMap, fmt::Debug};

use swc_atoms::Atom;

use crate::Ty;

// TODO: move to a separate module
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefId(pub usize);

impl DefId {
	pub fn new(id: usize) -> Self {
		DefId(id)
	}
}

impl Debug for DefId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Def({})", self.0)
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum Def {
	Func(Func),
	Class(Class),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Func {
	pub locals: HashMap<LocalId, Value>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Class {
	pub ctor: Option<Func>,
	pub methods: Vec<Func>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(usize);

impl LocalId {
	pub fn new(id: usize) -> Self {
		LocalId(id)
	}
}

impl Debug for LocalId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Local({})", self.0)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Local<'tcx> {
	pub id: LocalId,
	pub ty: Ty<'tcx>,
}

impl Debug for Local<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Local({}, {})", self.id.0, self.ty)
	}
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Value {
	Param,
	Ret,
	Var,
	// TODO: null?
	Err,
	Ref(DefId),
	Bool(bool),
	Int(i64),
	Str(Atom),
	Obj(Object),
	Array(Vec<LocalId>),
	Template(Vec<LocalId>),
	Regex(Atom),
	Call(LocalId, Vec<LocalId>),
	New(LocalId, Vec<LocalId>),
	Eq(LocalId, LocalId),
	TypeOf(LocalId),
	Closure(),
	Member(LocalId, Atom),
	Unary(UnaryOp, LocalId),
	Binary(BinaryOp, LocalId, LocalId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
	Not,
	Plus,
	Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
	Add,
	Sub,
	Mul,
	Div,
	Lt,
	LtEq,
	Gt,
	GtEq,
	And,
	Or,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Object {
	pub fields: Vec<(Atom, LocalId)>,
}

impl Object {
	pub fn new() -> Self {
		Self { fields: Vec::new() }
	}
}

impl Default for Object {
	fn default() -> Self {
		Self::new()
	}
}
