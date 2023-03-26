/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use std::{
	borrow::Borrow,
	collections::HashMap,
	fmt::{self, Display},
	ops::Deref,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[serde(transparent)]
pub struct TaskName(pub String);

pub type TaskNameMap = HashMap<usize, TaskName>;

impl From<String> for TaskName {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl Deref for TaskName {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.as_str()
	}
}

impl Borrow<str> for TaskName {
	fn borrow(&self) -> &str {
		self
	}
}

impl Display for TaskName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\"{}\"", self.0)
	}
}
