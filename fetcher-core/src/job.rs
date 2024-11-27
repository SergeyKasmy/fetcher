/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

pub mod timepoint;

use futures::future::join_all;
use tokio::time::sleep;

use self::timepoint::TimePoint;
use crate::{error::FetcherError, task::Task};

/// A single job, containing a single or a couple [`tasks`](`Task`), possibly refetching every set amount of time
#[derive(Debug)]
pub struct Job {
	/// The tasks to run
	pub tasks: Vec<Task>,

	/// Refresh/refetch/redo the job every "this" point of the day
	pub refresh_time: Option<TimePoint>,
}

impl Job {
	/// Run this job to completion or return early on an error
	///
	/// # Errors
	/// if any of the inner tasks return an error, refer to [`Task`] documentation
	pub async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		loop {
			let tasks = self.tasks.iter_mut().map(Task::run);
			let results = join_all(tasks).await;

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

			match &self.refresh_time {
				Some(refresh_time) => {
					let remaining_time = refresh_time.remaining_from_now();

					tracing::debug!(
						"Putting job to sleep for {}m",
						remaining_time.as_secs() / 60
					);
					sleep(remaining_time).await;
				}
				None => return Ok(()),
			}
		}
	}
}
