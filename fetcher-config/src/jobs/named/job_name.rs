/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{
	borrow::Borrow,
	fmt::{self, Display},
	ops::Deref,
	sync::Arc,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct JobName(pub Arc<str>);

impl JobName {
	#[must_use]
	pub fn as_str(&self) -> &str {
		self
	}
}

impl<T: Into<Arc<str>>> From<T> for JobName {
	fn from(value: T) -> Self {
		Self(value.into())
	}
}

impl Deref for JobName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Borrow<str> for JobName {
	fn borrow(&self) -> &str {
		self
	}
}

impl Display for JobName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\"{}\"", self.0)
	}
}
