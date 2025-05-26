/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

/// A string that is guaranteed to contain at least one element.
///
/// [`NonEmptyString`] provides a safe wrapper around `String` that maintains the invariant
/// that the string can never be empty.
///
/// # Examples
///
/// ```
/// use non_non_full::NonEmptyString;
///
/// let string = "Hello, World!".to_owned();
/// let non_empty = NonEmptyString::new(string).unwrap();
///
/// // Empty String returns None
/// assert!(NonEmptyString::new(String::new()).is_none());
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyString(String);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for NonEmptyString {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		let string = String::deserialize(deserializer)?;
		NonEmptyString::new(string).ok_or_else(|| {
			serde::de::Error::custom("cannot deserialize empty string as NonEmptyString")
		})
	}
}

impl NonEmptyString {
	/// Creates a new NonEmptyString from a String.
	/// Returns None if the input String is empty.
	pub fn new(string: String) -> Option<Self> {
		if string.is_empty() {
			None
		} else {
			Some(Self(string))
		}
	}

	/// Gets a reference to the underlying String
	pub fn as_string(&self) -> &String {
		&self.0
	}

	/// Converts the NonEmptyString back into a String, consuming self
	pub fn into_string(self) -> String {
		self.0
	}

	/// Returns a string slice containing the entire string
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}

	/// Pushes a char onto the end of the string
	pub fn push(&mut self, ch: char) {
		self.0.push(ch);
	}

	/// Pushes a string slice onto the end of the string
	pub fn push_str(&mut self, string: &str) {
		self.0.push_str(string);
	}

	/// Inserts a char at the given byte index
	pub fn insert(&mut self, idx: usize, ch: char) {
		self.0.insert(idx, ch);
	}

	/// Inserts a string slice at the given byte index
	pub fn insert_str(&mut self, idx: usize, string: &str) {
		self.0.insert_str(idx, string);
	}

	/// Removes the char at the given byte index.
	///
	/// Returns None if removing would make the string empty.
	pub fn remove(&mut self, idx: usize) -> Option<char> {
		if self.0.len() == 1 {
			None
		} else {
			Some(self.0.remove(idx))
		}
	}

	/// Removes the last char from the string.
	///
	/// Returns None if this would make the string empty.
	pub fn pop(&mut self) -> Option<char> {
		if self.0.len() == 1 {
			None
		} else {
			self.0.pop()
		}
	}

	/// Returns the length of the string in bytes
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Clears all chars except the first one
	pub fn clear_except_first(&mut self) {
		let first = self.0.remove(0);
		self.0.clear();
		self.0.push(first);
	}

	/// Returns an iterator over the chars in the string
	pub fn chars(&self) -> std::str::Chars<'_> {
		self.0.chars()
	}

	/// Returns an iterator over the char indices in the string
	pub fn char_indices(&self) -> std::str::CharIndices<'_> {
		self.0.char_indices()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_with_empty_string() {
		let empty = String::new();
		assert!(NonEmptyString::new(empty).is_none());
	}

	#[test]
	fn new_with_non_empty_string() {
		let non_empty = String::from("hello");
		let non_empty_string = NonEmptyString::new(non_empty.clone()).unwrap();
		assert_eq!(non_empty_string.as_string(), &non_empty);
	}

	#[test]
	fn push_and_pop() {
		let mut string = NonEmptyString::new(String::from("a")).unwrap();
		string.push('b');
		string.push('c');
		assert_eq!(string.len(), 3);
		assert_eq!(string.pop(), Some('c'));
		assert_eq!(string.pop(), Some('b'));
		assert_eq!(string.pop(), None);
	}

	#[test]
	fn remove() {
		let mut string = NonEmptyString::new(String::from("abc")).unwrap();
		assert_eq!(string.remove(1), Some('b'));
		assert_eq!(string.as_str(), "ac");
		assert_eq!(string.remove(0), Some('a'));
		assert_eq!(string.remove(0), None);
	}

	#[test]
	fn clear_except_first() {
		let mut string = NonEmptyString::new(String::from("abcd")).unwrap();
		string.clear_except_first();
		assert_eq!(string.as_str(), "a");
	}

	#[test]
	fn push_str() {
		let mut string = NonEmptyString::new(String::from("hello")).unwrap();
		string.push_str(" world");
		assert_eq!(string.as_str(), "hello world");
	}
}
