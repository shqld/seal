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

	pub fn name(&self) -> &Atom {
		&self.0.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_symbol_new() {
		let atom = Atom::new("test");
		let ctx = SyntaxContext::empty();
		let id = (atom.clone(), ctx);
		let symbol = Symbol::new(id);
		
		assert_eq!(symbol.name(), &atom);
	}

	#[test]
	fn test_symbol_new_main() {
		let symbol = Symbol::new_main();
		assert_eq!(symbol.name(), &Atom::new("@main"));
	}

	#[test]
	fn test_symbol_new_ret() {
		let symbol = Symbol::new_ret();
		assert_eq!(symbol.name(), &Atom::new("@ret"));
	}

	#[test]
	fn test_symbol_name() {
		let atom = Atom::new("variable");
		let symbol = Symbol::new((atom.clone(), SyntaxContext::empty()));
		assert_eq!(symbol.name(), &atom);
	}

	#[test]
	fn test_symbol_display() {
		let symbol = Symbol::new((Atom::new("myVar"), SyntaxContext::empty()));
		assert_eq!(format!("{}", symbol), "myVar");
	}

	#[test]
	fn test_symbol_debug() {
		let symbol = Symbol::new((Atom::new("testVar"), SyntaxContext::empty()));
		let debug_str = format!("{:?}", symbol);
		assert!(debug_str.contains("Symbol"));
		assert!(debug_str.contains("testVar"));
	}

	#[test]
	fn test_symbol_clone() {
		let symbol1 = Symbol::new((Atom::new("original"), SyntaxContext::empty()));
		let symbol2 = symbol1.clone();
		assert_eq!(symbol1.name(), symbol2.name());
	}

	#[test]
	fn test_symbol_eq() {
		let symbol1 = Symbol::new((Atom::new("same"), SyntaxContext::empty()));
		let symbol2 = Symbol::new((Atom::new("same"), SyntaxContext::empty()));
		let symbol3 = Symbol::new((Atom::new("different"), SyntaxContext::empty()));
		
		assert_eq!(symbol1, symbol2);
		assert_ne!(symbol1, symbol3);
	}

	#[test]
	fn test_symbol_ord() {
		let symbol_a = Symbol::new((Atom::new("a"), SyntaxContext::empty()));
		let symbol_b = Symbol::new((Atom::new("b"), SyntaxContext::empty()));
		
		assert!(symbol_a < symbol_b);
	}
}
