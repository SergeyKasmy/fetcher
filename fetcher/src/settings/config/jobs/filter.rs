/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use color_eyre::{eyre::eyre, Report};
use std::str::FromStr;

#[derive(Debug)]
pub struct JobFilter {
	pub job: String,
	pub task: Option<TaskFilter>,
}

#[derive(Debug)]
pub enum TaskFilter {
	All,
	Name(String),
	Id(usize),
}

impl JobFilter {
	#[must_use]
	pub fn job_matches(&self, job_name: &str) -> bool {
		self.job == job_name
	}

	#[must_use]
	pub fn task_matches_name(&self, job_name: &str, task_name: &str) -> bool {
		self.job == job_name
			&& self.task.as_ref().map_or(true, |task| {
				if let TaskFilter::Name(task) = task {
					task == task_name
				} else {
					false
				}
			})
	}

	#[must_use]
	pub fn task_matches_id(&self, job_name: &str, task_id: usize) -> bool {
		self.job == job_name
			&& self.task.as_ref().map_or(true, |task| {
				if let TaskFilter::Id(i) = task {
					*i == task_id
				} else {
					false
				}
			})
	}
}

impl FromStr for JobFilter {
	type Err = Report;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split(':').count() {
			1 => Ok(Self {
				job: s.to_owned(),
				task: None,
			}),
			2 => {
				let mut splits = s.split(':');

				let job = splits
					.next()
					.expect("should always exist since split count is 2, i.e. before and after")
					.to_owned();
				let task = splits
					.next()
					.expect("should always exist since split count is 2, i.e. before and after")
					.to_owned();

				Ok(Self {
					job,
					task: Some(match task.parse() {
						Ok(i) => TaskFilter::Id(i),
						Err(_) => TaskFilter::Name(task),
					}),
				})
			}
			_ => Err(eyre!(
				"\":\" can't be present more than once in a run filter"
			)),
		}
	}
}
