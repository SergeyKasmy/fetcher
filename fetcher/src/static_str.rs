use std::{borrow::Cow, fmt::Display, ops::Deref};

#[derive(Clone, Debug)]
pub struct StaticStr(Cow<'static, str>);

impl StaticStr {
	pub const fn from_static_str(s: &'static str) -> Self {
		Self(Cow::Borrowed(s))
	}

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
