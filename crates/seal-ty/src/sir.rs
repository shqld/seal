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

#[cfg(test)]
mod tests {
	use super::*;
	use swc_atoms::Atom;

	#[test]
	fn test_def_id_debug() {
		let id = DefId::new(42);
		assert_eq!(format!("{:?}", id), "Def(42)");
	}

	#[test]
	fn test_local_id_debug() {
		let id = LocalId::new(123);
		assert_eq!(format!("{:?}", id), "Local(123)");
	}

	#[test]
	fn test_object_new() {
		let obj = Object::new();
		assert_eq!(obj.fields.len(), 0);
	}

	#[test]
	fn test_object_default() {
		let obj = Object::default();
		assert_eq!(obj.fields.len(), 0);
	}

	#[test]
	fn test_value_variants() {
		let param = Value::Param;
		let ret = Value::Ret;
		let var = Value::Var;
		let err = Value::Err;

		assert_eq!(param, Value::Param);
		assert_eq!(ret, Value::Ret);
		assert_eq!(var, Value::Var);
		assert_eq!(err, Value::Err);
	}

	#[test]
	fn test_value_with_data() {
		let bool_val = Value::Bool(true);
		let int_val = Value::Int(42);
		let str_val = Value::Str(Atom::new("test"));
		let obj_val = Value::Obj(Object::new());

		assert_eq!(bool_val, Value::Bool(true));
		assert_eq!(int_val, Value::Int(42));
		assert_eq!(str_val, Value::Str(Atom::new("test")));
		assert_eq!(obj_val, Value::Obj(Object::new()));
	}

	#[test]
	fn test_unary_op_variants() {
		let not_op = UnaryOp::Not;
		let plus_op = UnaryOp::Plus;
		let minus_op = UnaryOp::Minus;

		assert_eq!(not_op, UnaryOp::Not);
		assert_eq!(plus_op, UnaryOp::Plus);
		assert_eq!(minus_op, UnaryOp::Minus);
	}

	#[test]
	fn test_binary_op_variants() {
		let add_op = BinaryOp::Add;
		let sub_op = BinaryOp::Sub;
		let mul_op = BinaryOp::Mul;
		let div_op = BinaryOp::Div;
		let lt_op = BinaryOp::Lt;
		let lteq_op = BinaryOp::LtEq;
		let gt_op = BinaryOp::Gt;
		let gteq_op = BinaryOp::GtEq;
		let and_op = BinaryOp::And;
		let or_op = BinaryOp::Or;

		assert_eq!(add_op, BinaryOp::Add);
		assert_eq!(sub_op, BinaryOp::Sub);
		assert_eq!(mul_op, BinaryOp::Mul);
		assert_eq!(div_op, BinaryOp::Div);
		assert_eq!(lt_op, BinaryOp::Lt);
		assert_eq!(lteq_op, BinaryOp::LtEq);
		assert_eq!(gt_op, BinaryOp::Gt);
		assert_eq!(gteq_op, BinaryOp::GtEq);
		assert_eq!(and_op, BinaryOp::And);
		assert_eq!(or_op, BinaryOp::Or);
	}

	#[test]
	fn test_def_variants() {
		let func = Def::Func(Func {
			locals: HashMap::new(),
		});
		let class = Def::Class(Class {
			ctor: None,
			methods: Vec::new(),
		});

		match func {
			Def::Func(_) => {}
			_ => panic!("Expected Func variant"),
		}

		match class {
			Def::Class(_) => {}
			_ => panic!("Expected Class variant"),
		}
	}

	#[test]
	fn test_class_with_constructor() {
		let class = Class {
			ctor: Some(Func {
				locals: HashMap::new(),
			}),
			methods: Vec::new(),
		};

		assert!(class.ctor.is_some());
		assert_eq!(class.methods.len(), 0);
	}

	#[test]
	fn test_object_with_fields() {
		let mut obj = Object::new();
		obj.fields.push((Atom::new("name"), LocalId::new(1)));
		obj.fields.push((Atom::new("age"), LocalId::new(2)));

		assert_eq!(obj.fields.len(), 2);
		assert_eq!(obj.fields[0].0, Atom::new("name"));
		assert_eq!(obj.fields[1].0, Atom::new("age"));
	}
}
