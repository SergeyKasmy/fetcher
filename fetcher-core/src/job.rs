/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

use futures::future::join_all;
use std::time::Duration;
use tokio::time::sleep;

use crate::{error::Error, task::Task};

/// A single job, containing a single or a couple [`tasks`](`Task`), possibly refetching every set amount of time
#[derive(Debug)]
pub struct Job {
	/// The tasks to run
	pub tasks: Vec<Task>,

	/// Refetch every interval or just run once
	pub refetch_interval: Option<Duration>,
}

impl Job {
	/// Run this job to completion or return early on an error
	///
	/// # Errors
	/// if any of the inner tasks return an error, refer to [`Task`] documentation
	pub async fn run(&mut self) -> Result<(), Vec<Error>> {
		loop {
			let jobs = self.tasks.iter_mut().map(Task::run);
			let results = join_all(jobs).await;

			let errors = results
				.into_iter()
				.filter_map(|r| match r {
					Ok(()) => None,
					Err(e) => Some(e),
				})
				.collect::<Vec<_>>();

			if !errors.is_empty() {
				return Err(errors);
			}

			match self.refetch_interval {
				Some(refetch_interval) => {
					tracing::debug!(
						"Putting job to sleep for {}m",
						refetch_interval.as_secs() / 60
					);
					sleep(refetch_interval).await;
				}
				None => return Ok(()),
			}
		}
	}
}
