use std::{borrow::Cow, fmt::Display, ops::Deref};

/// A string that has a 'static lifetime.
///
/// This makes it possible to use [`&'static str`]'s directly without allocating
/// while also allowing the use of plain regular-old [`String`]s.
/// This is most useful in places where in 99% of times a [`&'static str`] is used but sometimes a [`format!()`]'ed string may be required.
/// Technically, this could be used everywhere instead of [`String`]s but this introduces too much boilerplate and `.into()` transitions
/// that just pollute the code for little benefit.
// TODO: this could technically just be replaced with a smolstr of something of this sort to avoid a enum branch. (is this even an issue in this type of program?)
#[derive(Clone, Debug)]
pub struct StaticStr(Cow<'static, str>);

impl StaticStr {
	#[must_use]
	pub const fn from_static_str(s: &'static str) -> Self {
		Self(Cow::Borrowed(s))
	}

	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl Deref for StaticStr {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
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

impl Display for StaticStr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(self)
	}
}

impl AsRef<str> for StaticStr {
	fn as_ref(&self) -> &str {
		self.0.as_ref()
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
