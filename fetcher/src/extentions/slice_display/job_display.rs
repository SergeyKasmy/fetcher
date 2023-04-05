/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the adapter that implements `Display` for ([`JobName`], [`JobWithTaskNames`])
//! intended to format jobs and tasks as "JOB NAME":["TASK1", "TASK2"]

use fetcher_config::jobs::named::{JobName, JobWithTaskNames};

use std::{cmp, fmt};

pub struct JobDisplay<'a>(pub (&'a JobName, &'a JobWithTaskNames));

impl fmt::Display for JobDisplay<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let JobDisplay((
			job_name,
			JobWithTaskNames {
				inner: _,
				task_names,
			},
		)) = self;

		write!(f, "{job_name}")?;

		if let Some(task_names) = task_names {
			assert!(
				!task_names.is_empty(),
				"Developer error: Task names map should either contain names or be None"
			);

			f.write_str(":[")?;

			let mut task_names = task_names.values();

			write!(
				f,
				"{}",
				task_names
					.next()
					.expect("should contains at least one item because of the assert above")
			)?;

			for name in task_names {
				write!(f, ", {name}")?;
			}

			f.write_str("]")?;
		}

		Ok(())
	}
}

impl cmp::PartialOrd for JobDisplay<'_> {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.0 .0.as_str().partial_cmp(other.0 .0.as_str())
	}
}

impl cmp::Ord for JobDisplay<'_> {
	fn cmp(&self, other: &Self) -> cmp::Ordering {
		self.0 .0.as_str().cmp(other.0 .0.as_str())
	}
}

impl cmp::PartialEq for JobDisplay<'_> {
	fn eq(&self, other: &Self) -> bool {
		self.0 .0 == other.0 .0
	}
}

impl cmp::Eq for JobDisplay<'_> {}
