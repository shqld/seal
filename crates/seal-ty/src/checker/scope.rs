use std::cell::Cell;

/// NOTE: This Scope is not for syntax but for types.
///       For example, a Block statement does not create a new scope, whereas a ternary expression does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TyScope(usize);

impl TyScope {
	pub fn root() -> Self {
		Self(0)
	}

	pub fn next(&self) -> Self {
		Self(self.0 + 1)
	}
}

#[derive(Debug)]
pub struct ScopeStack {
	inner: Vec<TyScope>,
	// NOTE: Seal needs the counter to avoid overriding previously scoped types
	counter: Cell<TyScope>,
}

impl ScopeStack {
	pub fn new() -> Self {
		let root = TyScope::root();
		Self {
			inner: vec![root],
			counter: Cell::new(root),
		}
	}

	pub fn push(&mut self) -> TyScope {
		let next = self.counter.get().next();

		self.counter.set(next);
		self.inner.push(next);

		next
	}

	pub fn pop(&mut self) {
		self.inner.pop();
	}

	pub fn peek(&self) -> TyScope {
		*self.inner.last().unwrap()
	}
}
