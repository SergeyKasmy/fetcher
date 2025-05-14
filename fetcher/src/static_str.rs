use std::{
	borrow::{Borrow, Cow},
	fmt::Display,
	ops::Deref,
};

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct StaticStr(Cow<'static, str>);

impl StaticStr {
	pub const fn from_static_str(s: &'static str) -> Self {
		Self(Cow::Borrowed(s))
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn is_empty(&self) -> bool {
		self.as_str().is_empty()
	}

	pub fn into_heap_allocated(&mut self) -> &mut String {
		if let Cow::Borrowed(borrowed_str) = self.0 {
			self.0 = Cow::Owned(borrowed_str.to_owned());
		}

		let Cow::Owned(heap_allocated_str) = &mut self.0 else {
			unreachable!();
		};

		heap_allocated_str
	}
}

impl Deref for StaticStr {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl AsRef<str> for StaticStr {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl Borrow<str> for StaticStr {
	fn borrow(&self) -> &str {
		&self.0
	}
}

impl FromIterator<StaticStr> for StaticStr {
	fn from_iter<T: IntoIterator<Item = StaticStr>>(iter: T) -> Self {
		let mut iter = iter.into_iter();

		match iter.next() {
			None => StaticStr::from_static_str(""),
			Some(mut buf) => {
				buf.extend(iter);
				buf
			}
		}
	}
}

impl Extend<StaticStr> for StaticStr {
	fn extend<T: IntoIterator<Item = StaticStr>>(&mut self, iter: T) {
		let heap_allocated_str = self.into_heap_allocated();

		for static_str in iter {
			heap_allocated_str.push_str(&static_str);
		}
	}
}

impl Default for StaticStr {
	fn default() -> Self {
		Self::from_static_str("")
	}
}

impl From<StaticStr> for String {
	fn from(value: StaticStr) -> Self {
		value.0.into_owned()
	}
}

impl From<&StaticStr> for String {
	fn from(value: &StaticStr) -> Self {
		value.as_str().to_owned()
	}
}

impl From<String> for StaticStr {
	fn from(value: String) -> Self {
		Self(Cow::Owned(value))
	}
}

impl From<&'static str> for StaticStr {
	fn from(value: &'static str) -> Self {
		Self(Cow::Borrowed(value))
	}
}

impl From<Cow<'_, str>> for StaticStr {
	fn from(value: Cow<'_, str>) -> Self {
		match value {
			Cow::Borrowed(v) => v.to_owned().into(),
			Cow::Owned(v) => v.into(),
		}
	}
}

impl Display for StaticStr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self)
	}
}
