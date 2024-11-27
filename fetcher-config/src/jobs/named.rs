/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod job_name;
mod task_name;

pub use self::{
	job_name::JobName,
	task_name::{TaskName, TaskNameMap},
};

use fetcher_core::job::Job;

use std::collections::HashMap;

#[derive(Debug)]
pub struct JobWithTaskNames {
	pub inner: Job,
	pub task_names: Option<HashMap<usize, TaskName>>,
}
