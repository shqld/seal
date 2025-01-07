use swc_atoms::Atom;
use swc_common::SyntaxContext;
use swc_ecma_ast::Id;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Symbol(Id);

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
