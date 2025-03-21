/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

pub mod error_handling;
pub mod opaque_job;

mod job_group;
mod timepoint;

pub use self::{
	error_handling::ErrorHandling, job_group::JobGroup, opaque_job::OpaqueJob, timepoint::TimePoint,
};

use std::ops::ControlFlow;
use tokio::time::sleep;

use crate::{
	StaticStr,
	error::{ErrorChainDisplay, FetcherError},
	task::TaskGroup,
};

/// A single job, containing a single or a couple [`tasks`](`Task`), possibly refetching every set amount of time
#[derive(bon::Builder, Debug)]
pub struct Job<T> {
	#[builder(start_fn, into)]
	pub name: StaticStr,

	/// The tasks to run
	pub tasks: T,

	/// Refresh/refetch/redo the job every "this" point of the day
	pub refresh_time: Option<TimePoint>,

	/// What to do incase an error occures?
	#[builder(default)]
	pub error_handling: ErrorHandling,
}

impl<T: TaskGroup> Job<T> {
	/// Run this job to completion or return early on an error
	///
	/// # Errors
	/// if any of the inner tasks return an error, refer to [`Task`] documentation
	pub async fn run_without_error_handling(&mut self) -> Result<(), Vec<FetcherError>> {
		loop {
			let results = self.tasks.run_concurrently().await;

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

impl<T: TaskGroup> OpaqueJob for Job<T> {
	/// Run this job to completion or return early on an error
	///
	/// # Errors
	/// if any of the inner tasks return an error, refer to [`Task`] documentation
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		loop {
			let job_result = self.run_without_error_handling().await;

			match self
				.error_handling
				.handle_job_result(job_result, &self.name, self.refresh_time.as_ref())
				.await
			{
				ControlFlow::Continue(()) => (),
				ControlFlow::Break(res) => return res,
			}
		}
	}

	fn name(&self) -> Option<&str> {
		Some(&self.name)
	}
}
