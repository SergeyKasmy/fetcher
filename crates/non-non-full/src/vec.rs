#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyVec<T>(Vec<T>);

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

impl<T> NonEmptyVec<T> {
	/// Creates a new NonEmptyVec from a Vec.
	///
	/// Returns None if the input Vec is empty.
	pub fn new(vec: Vec<T>) -> Option<Self> {
		if vec.is_empty() {
			None
		} else {
			Some(Self(vec))
		}
	}

	/// Gets a reference to the underlying Vec
	pub fn as_vec(&self) -> &Vec<T> {
		&self.0
	}

	/// Converts the NonEmptyVec back into a Vec, consuming self
	pub fn into_vec(self) -> Vec<T> {
		self.0
	}

	/// Returns a slice containing the entire vector
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

	/// Returns the length of the vector
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Gets a reference to an element at the given index
	pub fn get(&self, index: usize) -> Option<&T> {
		self.0.get(index)
	}

	/// Gets a mutable reference to an element at the given index
	pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
		self.0.get_mut(index)
	}

	/// Returns a reference to the first element
	pub fn first(&self) -> &T {
		self.0
			.first()
			.expect("NonEmptyVec invariant guarantees at least one element")
	}

	/// Returns a mutable reference to the first element
	pub fn first_mut(&mut self) -> &mut T {
		self.0
			.first_mut()
			.expect("NonEmptyVec invariant guarantees at least one element")
	}

	/// Returns a reference to the last element
	pub fn last(&self) -> &T {
		self.0
			.last()
			.expect("NonEmptyVec invariant guarantees at least one element")
	}

	/// Returns a mutable reference to the last element
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
}
