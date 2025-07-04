use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash};

use typed_arena::Arena;

use super::interned::Interned;

pub struct Interner<'a, T: Hash + PartialEq + Eq> {
	arena: Arena<T>,
	map: RefCell<HashMap<&'a T, Interned<'a, T>>>,
}

impl<'a, T: Hash + PartialEq + Eq> Interner<'a, T> {
	pub fn new() -> Self {
		Interner {
			arena: Arena::new(),
			map: RefCell::new(HashMap::new()),
		}
	}

	pub fn intern(&'a self, t: T) -> Interned<'a, T> {
		let mut map = self.map.borrow_mut();

		if let Some(ty) = map.get(&t) {
			*ty
		} else {
			let id = self.arena.len();
			let ref_ = self.arena.alloc(t);
			let interned = Interned::new(ref_, id);

			map.insert(ref_, interned);

			interned
		}
	}
}

impl<T: Hash + PartialEq + Eq> Default for Interner<'_, T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Debug + Hash + PartialEq + Eq> Debug for Interner<'_, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut f = f.debug_list();

		for (k, _) in self.map.borrow().iter() {
			f.entry(k);
		}

		f.finish()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_interner_new() {
		let interner: Interner<String> = Interner::new();
		assert_eq!(interner.arena.len(), 0);
		assert_eq!(interner.map.borrow().len(), 0);
	}

	#[test]
	fn test_interner_default() {
		let interner: Interner<String> = Interner::default();
		assert_eq!(interner.arena.len(), 0);
		assert_eq!(interner.map.borrow().len(), 0);
	}

	#[test]
	fn test_interner_intern_new_value() {
		let interner = Interner::new();
		let interned = interner.intern("hello".to_string());

		assert_eq!(interned.id(), 0);
		assert_eq!(*interned.get(), "hello");
		assert_eq!(interner.arena.len(), 1);
		assert_eq!(interner.map.borrow().len(), 1);
	}

	#[test]
	fn test_interner_intern_duplicate_value() {
		let interner = Interner::new();
		let first = interner.intern("hello".to_string());
		let second = interner.intern("hello".to_string());

		// Should return the same interned value
		assert_eq!(first.id(), second.id());
		assert_eq!(first.get(), second.get());
		assert_eq!(interner.arena.len(), 1); // Still only one item in arena
		assert_eq!(interner.map.borrow().len(), 1); // Still only one item in map
	}

	#[test]
	fn test_interner_intern_multiple_values() {
		let interner = Interner::new();
		let hello = interner.intern("hello".to_string());
		let world = interner.intern("world".to_string());
		let hello2 = interner.intern("hello".to_string());

		// Different strings get different IDs
		assert_ne!(hello.id(), world.id());

		// Same string gets same ID
		assert_eq!(hello.id(), hello2.id());

		// Check arena and map sizes
		assert_eq!(interner.arena.len(), 2);
		assert_eq!(interner.map.borrow().len(), 2);
	}

	#[test]
	fn test_interner_debug() {
		let interner = Interner::new();
		interner.intern(42);
		interner.intern(100);

		let debug_string = format!("{:?}", interner);
		// Should be a debug list containing the interned values
		assert!(debug_string.starts_with("["));
		assert!(debug_string.ends_with("]"));
	}

	#[test]
	fn test_interner_with_integers() {
		let interner = Interner::new();
		let first = interner.intern(42);
		let second = interner.intern(42);
		let third = interner.intern(100);

		assert_eq!(first.id(), second.id());
		assert_ne!(first.id(), third.id());
		assert_eq!(*first.get(), 42);
		assert_eq!(*third.get(), 100);
	}
}
