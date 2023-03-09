/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod action;
pub mod external_data;
pub mod job;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;

pub use self::job::Job;

use core::fmt;
use std::ops::Deref;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JobName(pub String);

#[derive(Clone, Debug)]
pub enum TaskId {
	Name(String),
	Id(usize),
}

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

impl fmt::Display for JobName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl fmt::Display for TaskId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			TaskId::Name(s) => f.write_str(s),
			TaskId::Id(i) => write!(f, "{i}"),
		}
	}
}
