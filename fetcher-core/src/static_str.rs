use std::{borrow::Cow, fmt::Display, ops::Deref};

#[derive(Clone, Debug)]
pub struct StaticStr(Cow<'static, str>);

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
