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
