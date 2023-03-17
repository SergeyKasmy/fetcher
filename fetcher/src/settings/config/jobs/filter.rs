/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use color_eyre::{eyre::eyre, Report};
use fetcher_config::jobs::{JobName, TaskName};
use std::str::FromStr;

#[derive(Debug)]
pub struct JobFilter {
	pub job: JobName,
	pub task: Option<TaskName>,
}

// TODO: compare in lower case
impl JobFilter {
	#[must_use]
	pub fn job_matches(&self, job_name: &JobName) -> bool {
		&self.job == job_name
	}

	#[must_use]
	pub fn task_matches(&self, job_name: &JobName, task_name: &TaskName) -> bool {
		&self.job == job_name
			&& self
				.task
				.as_ref()
				.map_or(true, |task_filter| task_filter == task_name)
	}
}

impl FromStr for JobFilter {
	type Err = Report;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split(':').count() {
			1 => Ok(Self {
				job: s.to_owned().into(),
				task: None,
			}),
			2 => {
				let mut splits = s.split(':');

				let job = splits
					.next()
					.expect("should always exist since split count is 2, i.e. before and after")
					.to_owned()
					.into();
				let task = splits
					.next()
					.expect("should always exist since split count is 2, i.e. before and after")
					.to_owned()
					.into();

				Ok(Self {
					job,
					task: Some(task),
				})
			}
			_ => Err(eyre!(
				"\":\" can't be present more than once in a run filter"
			)),
		}
	}
}
