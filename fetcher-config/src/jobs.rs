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
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Deref};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Tag {
	String(String),
	UseTaskName,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JobName(pub String);

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[serde(transparent)]
pub struct TaskName(pub String);

pub type TaskNameMap = HashMap<usize, TaskName>;

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

impl fmt::Display for TaskName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}
