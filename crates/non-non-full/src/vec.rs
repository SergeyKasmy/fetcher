/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`NonEmptyVec`] type.

use std::ops;

/// A vector that is guaranteed to contain at least one element.
///
/// `NonEmptyVec` provides a safe wrapper around `Vec` that maintains the invariant
/// that the vector can never be empty. This is useful when you need to ensure that
/// a collection always has at least one element, making it impossible to represent
/// an empty state.
///
/// # Examples
///
/// ```
/// use non_non_full::NonEmptyVec;
///
/// // Create a new non-empty vector
/// let vec = NonEmptyVec::new_one(42);
///
/// // Attempt to create from an existing Vec
/// let vec = vec![1, 2, 3];
/// let non_empty = NonEmptyVec::new(vec).unwrap();
///
/// // Empty Vec returns None
/// assert!(NonEmptyVec::new(Vec::<()>::new()).is_none());
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyVec<T>(Vec<T>);

impl<T> std::hash::Hash for NonEmptyVec<T>
where
	T: std::hash::Hash,
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl<T> NonEmptyVec<T> {
	/// Creates a new [`NonEmptyVec`] from a [`Vec`].
	///
	/// Returns `None` if the input [`Vec`] is empty.
	#[must_use]
	pub fn new(vec: Vec<T>) -> Option<Self> {
		if vec.is_empty() {
			None
		} else {
			Some(Self(vec))
		}
	}

	/// Creates a new [`NonEmptyVec`] containing exactly one element
	#[must_use]
	pub fn with_first(value: T) -> Self {
		Self(vec![value])
	}

	/// Returns a reference to the underlying [`Vec`]
	#[must_use]
	pub fn as_vec(&self) -> &Vec<T> {
		&self.0
	}

	/// Converts the [`NonEmptyVec`] back into a [`Vec`], consuming self
	#[must_use]
	pub fn into_vec(self) -> Vec<T> {
		self.0
	}

	/// Returns a slice containing the entire vector
	#[must_use]
	pub fn as_slice(&self) -> &[T] {
		self.0.as_slice()
	}

	/// Appends an element onto the end of the vector
	pub fn push(&mut self, value: T) {
		self.0.push(value);
	}

	/// Inserts an element at the given index, shifting all elements after it to the right
	pub fn insert(&mut self, index: usize, value: T) {
		self.0.insert(index, value);
	}

	/// Removes and returns the element at the given index.
	///
	/// Returns None if removing would make the vector empty.
	pub fn remove(&mut self, index: usize) -> Option<T> {
		if self.0.len() == 1 {
			None
		} else {
			Some(self.0.remove(index))
		}
	}

	/// Pops the last element from the vector.
	///
	/// Returns None if this would make the vector empty.
	pub fn pop(&mut self) -> Option<T> {
		if self.0.len() == 1 {
			None
		} else {
			self.0.pop()
		}
	}

	/// Creates a new [`NonEmptyVec`] where each element is mapped via the provided closure.
	///
	/// This is an alternative to the ubiquitous `vec.into_iter().map().collect::<Vec<_>>()`
	/// as [`FromIterator`] can't be implemented for [`NonEmptyVec`]
	#[must_use]
	pub fn map<F, U>(self, f: F) -> NonEmptyVec<U>
	where
		F: FnMut(T) -> U,
	{
		NonEmptyVec(self.into_iter().map(f).collect())
	}

	/// Returns the length of the vector
	#[expect(clippy::len_without_is_empty, reason = "NonEmptyVec is never empty")] // it's literally in the name!
	#[must_use]
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Gets a reference to an element at the given index
	#[must_use]
	pub fn get(&self, index: usize) -> Option<&T> {
		self.0.get(index)
	}

	/// Gets a mutable reference to an element at the given index
	pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
		self.0.get_mut(index)
	}

	/// Returns a reference to the first element
	#[expect(clippy::missing_panics_doc)]
	#[must_use]
	pub fn first(&self) -> &T {
		self.0
			.first()
			.expect("NonEmptyVec invariant guarantees at least one element")
	}

	/// Returns a mutable reference to the first element
	#[expect(clippy::missing_panics_doc)]
	#[must_use]
	pub fn first_mut(&mut self) -> &mut T {
		self.0
			.first_mut()
			.expect("NonEmptyVec invariant guarantees at least one element")
	}

	/// Returns the first element, consuming the vector
	///
	/// Mostly useful when you are sure there is only one element
	/// or when you only need the one element and don't care about others
	///
	/// # Note
	/// Drops the other elements
	///
	/// # Example
	/// ```
	/// let non_empty = NonEmptyVec::new(vec![1, 2, 3]).unwrap();
	/// let first = non_empty.into_first();
	/// assert_eq!(first, 1);
	/// ```
	#[must_use]
	pub fn into_first(mut self) -> T {
		self.0.swap_remove(0)
	}

	/// Returns a reference to the last element
	#[expect(clippy::missing_panics_doc)]
	#[must_use]
	pub fn last(&self) -> &T {
		self.0
			.last()
			.expect("NonEmptyVec invariant guarantees at least one element")
	}

	/// Returns a mutable reference to the last element
	#[expect(clippy::missing_panics_doc)]
	#[must_use]
	pub fn last_mut(&mut self) -> &mut T {
		self.0
			.last_mut()
			.expect("NonEmptyVec invariant guarantees at least one element")
	}

	/// Clears all elements except the first one
	pub fn clear_except_first(&mut self) {
		let first = self.0.remove(0);
		self.0.clear();
		self.0.push(first);
	}

	/// Extends the vector with the contents of an iterator
	pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
		self.0.extend(iter);
	}
}

impl<T> IntoIterator for NonEmptyVec<T> {
	type Item = T;
	type IntoIter = std::vec::IntoIter<T>;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<'a, T> IntoIterator for &'a NonEmptyVec<T> {
	type Item = &'a T;
	type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.as_vec().iter()
	}
}

impl<'a, T> IntoIterator for &'a mut NonEmptyVec<T> {
	type Item = &'a mut T;
	type IntoIter = <&'a mut Vec<T> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter_mut()
	}
}

impl<T> ops::Deref for NonEmptyVec<T> {
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for NonEmptyVec<T>
where
	T: serde::Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let vec = Vec::deserialize(deserializer)?;
		NonEmptyVec::new(vec).ok_or_else(|| {
			serde::de::Error::custom("cannot deserialize empty vector as NonEmptyVec")
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_with_empty_vec() {
		let empty: Vec<i32> = vec![];
		assert!(NonEmptyVec::new(empty).is_none());
	}

	#[test]
	fn new_with_non_empty_vec() {
		let non_empty = vec![1, 2, 3];
		let non_empty_vec = NonEmptyVec::new(non_empty.clone()).unwrap();
		assert_eq!(non_empty_vec.as_vec(), &non_empty);
	}

	#[test]
	fn push_and_pop() {
		let mut vec = NonEmptyVec::new(vec![1]).unwrap();
		vec.push(2);
		vec.push(3);
		assert_eq!(vec.len(), 3);
		assert_eq!(vec.pop(), Some(3));
		assert_eq!(vec.pop(), Some(2));
		assert_eq!(vec.pop(), None);
	}

	#[test]
	fn remove() {
		let mut vec = NonEmptyVec::new(vec![1, 2, 3]).unwrap();
		assert_eq!(vec.remove(1), Some(2));
		assert_eq!(vec.as_vec(), &vec![1, 3]);
		assert_eq!(vec.remove(0), Some(1));
		assert_eq!(vec.remove(0), None);
	}

	#[test]
	fn first_last() {
		let mut vec = NonEmptyVec::new(vec![1, 2, 3]).unwrap();
		assert_eq!(vec.first(), &1);
		assert_eq!(vec.last(), &3);
		*vec.first_mut() = 10;
		*vec.last_mut() = 30;
		assert_eq!(vec.as_vec(), &vec![10, 2, 30]);
	}

	#[test]
	fn clear_except_first() {
		let mut vec = NonEmptyVec::new(vec![1, 2, 3, 4]).unwrap();
		vec.clear_except_first();
		assert_eq!(vec.as_vec(), &vec![1]);
	}

	#[test]
	fn with_first() {
		let vec = NonEmptyVec::with_first(42);
		assert_eq!(vec.len(), 1);
		assert_eq!(vec.first(), &42);
		assert_eq!(vec.as_vec(), &vec![42]);
	}
}
