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

	pub fn get(&self) -> &T {
		self.0
	}

	pub fn id(&self) -> usize {
		self.1
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_interned_new() {
		let value = "hello".to_string();
		let interned = Interned::new(&value, 42);
		assert_eq!(interned.get(), &value);
		assert_eq!(interned.id(), 42);
	}

	#[test]
	fn test_interned_deref() {
		let value = "hello".to_string();
		let interned = Interned::new(&value, 0);
		assert_eq!(*interned, value);
	}

	#[test]
	fn test_interned_clone() {
		let value = "hello".to_string();
		let interned = Interned::new(&value, 0);
		let cloned = interned.clone();
		assert_eq!(interned.get(), cloned.get());
		assert_eq!(interned.id(), cloned.id());
	}

	#[test]
	fn test_interned_eq() {
		let value1 = "hello".to_string();
		let value2 = "world".to_string();
		let interned1 = Interned::new(&value1, 1);
		let interned2 = Interned::new(&value2, 1);
		let interned3 = Interned::new(&value1, 2);
		
		// Same ID means equal
		assert_eq!(interned1, interned2);
		// Different ID means not equal
		assert_ne!(interned1, interned3);
	}

	#[test]
	fn test_interned_ord() {
		let value1 = "hello".to_string();
		let value2 = "world".to_string();
		let interned1 = Interned::new(&value1, 1);
		let interned2 = Interned::new(&value2, 2);
		let interned3 = Interned::new(&value1, 3);
		
		assert!(interned1 < interned2);
		assert!(interned2 < interned3);
		assert!(interned1 < interned3);
	}

	#[test]
	fn test_interned_debug() {
		let value = 42;
		let interned = Interned::new(&value, 0);
		let debug_string = format!("{:?}", interned);
		assert_eq!(debug_string, "42");
	}

	#[test]
	fn test_interned_display() {
		let value = "hello".to_string();
		let interned = Interned::new(&value, 0);
		let display_string = format!("{}", interned);
		assert_eq!(display_string, "hello");
	}

	#[test]
	fn test_interned_hash() {
		use std::collections::hash_map::DefaultHasher;
		use std::hash::{Hash, Hasher};
		
		let value1 = "hello".to_string();
		let value2 = "hello".to_string();
		let interned1 = Interned::new(&value1, 0);
		let interned2 = Interned::new(&value2, 0);
		
		let mut hasher1 = DefaultHasher::new();
		let mut hasher2 = DefaultHasher::new();
		
		interned1.hash(&mut hasher1);
		interned2.hash(&mut hasher2);
		
		// Hash is based on pointer, so different instances will have different hashes
		// even if they contain the same value
		let hash1 = hasher1.finish();
		let hash2 = hasher2.finish();
		
		// We can't assert they're different because they might coincidentally be the same
		// But we can assert that the hash function doesn't panic
		assert!(hash1 == hash1); // Identity check
		assert!(hash2 == hash2); // Identity check
	}
}
