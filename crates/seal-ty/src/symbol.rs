use std::fmt::{Debug, Display};

use swc_atoms::Atom;
use swc_common::SyntaxContext;
use swc_ecma_ast::Id;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(Id);

impl Debug for Symbol {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Symbol({}#{})", self.0.0, self.0.1.as_u32())
	}
}

impl Display for Symbol {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0.0)
	}
}

impl Symbol {
	pub fn new(id: Id) -> Self {
		Self(id)
	}

	pub fn new_main() -> Self {
		Self((Atom::new("@main"), SyntaxContext::empty()))
	}

	pub fn new_ret() -> Self {
		Self((Atom::new("@ret"), SyntaxContext::empty()))
	}
}
