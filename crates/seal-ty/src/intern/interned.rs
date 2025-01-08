use std::{
	fmt::{Debug, Display},
	hash::{Hash, Hasher},
	ops::Deref,
	ptr,
};

pub struct Interned<'a, T>(pub &'a T, pub usize);

impl<T> Clone for Interned<'_, T> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<T> Copy for Interned<'_, T> {}

impl<T> Deref for Interned<'_, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<T: Debug> Debug for Interned<'_, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl<T: Display> Display for Interned<'_, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl<T> Hash for Interned<'_, T> {
	#[inline]
	fn hash<H: Hasher>(&self, s: &mut H) {
		// Pointer hashing is sufficient, due to the uniqueness constraint.
		ptr::hash(self.0, s)
	}
}

impl<T> PartialEq for Interned<'_, T> {
	fn eq(&self, other: &Self) -> bool {
		self.1 == other.1
	}
}

impl<T> Eq for Interned<'_, T> {}

impl<T> PartialOrd for Interned<'_, T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.1.cmp(&other.1))
	}
}

impl<T> Ord for Interned<'_, T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.1.cmp(&other.1)
	}
}

impl<'a, T> Interned<'a, T> {
	pub fn new(t: &'a T, id: usize) -> Self {
		Interned(t, id)
	}
}
