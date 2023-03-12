/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{
	borrow::Borrow,
	fmt::{self, Display},
	ops::Deref,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JobName(pub String);

impl From<String> for JobName {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl Deref for JobName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.as_str()
	}
}

impl Borrow<str> for JobName {
	fn borrow(&self) -> &str {
		self
	}
}

impl Display for JobName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}
