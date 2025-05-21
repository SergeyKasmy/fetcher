/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

pub mod error_handling;
pub mod job_group;
pub mod opaque_job;
pub mod timepoint;

mod job_result;

pub use self::{
	error_handling::HandleError, job_group::JobGroup, job_result::JobResult, opaque_job::OpaqueJob,
	timepoint::TimePoint,
};

use error_handling::{HandleErrorContext, HandleErrorResult};
use tokio::{select, time::sleep};

use crate::{
	StaticStr,
	ctrl_c_signal::{CtrlCSignalChannel, ctrlc_signaled},
	error::ErrorChainDisplay,
	task::TaskGroup,
};

/// A single job, containing a single or a couple [`tasks`](`crate::task::Task`), possibly refetching every set amount of time
#[derive(bon::Builder, Debug)]
pub struct Job<T, H> {
	#[builder(start_fn, into)]
	pub name: StaticStr,

	/// The tasks to run
	pub tasks: T,

	/// Refresh/refetch/redo the job every "this" point of the day
	#[builder(required)]
	pub refresh_time: Option<TimePoint>,

	/// What to do incase an error occures?
	pub error_handling: H,

	/// Gracefully stop the job on a Ctrl-C
	#[builder(required)]
	pub ctrlc_chan: Option<CtrlCSignalChannel>,
}

impl<T: TaskGroup, H> Job<T, H> {
	// TODO: instead of returning a vec of errors, return a single error type with a pretty Display implementation
	// that contains a list of errors that can be retrieved manually if needed instead
	/// Run this job to completion or return early on an error.
	///
	/// # Errors
	/// if any of the inner tasks return an error, refer to [`Task`](`crate::task::Task`) documentation
	///
	/// # Note
	/// If you are a user of the library and want your job to stop as soon as any error occures,
	/// set error handling to [`error_handling::Forward`] and just run the job as normal.
	#[tracing::instrument(skip_all, fields(name = %self.name))]
	async fn run_until_first_error(&mut self) -> JobResult {
		tracing::info!("Running job {}", self.name);

		// Job loop: break out of it only on errors or if the job doesn't have a refresh time/runs only once
		loop {
			let results = self.tasks.run_concurrently().await;

			#[expect(clippy::manual_ok_err, reason = "false positive")]
			let errors = results
				.into_iter()
				.filter_map(|r| match r {
					Ok(()) => None,
					Err(e) => Some(e),
				})
				.collect::<Vec<_>>();

			// returns errors if any
			if !errors.is_empty() {
				return JobResult::Err(errors);
			}

			// stop the job if there's no refresh timer
			let Some(refresh_time) = &self.refresh_time else {
				return JobResult::Ok;
			};

			let remaining_time = refresh_time.remaining_from_now();

			tracing::debug!(
				"Putting job to sleep for {}m",
				remaining_time.as_secs() / 60
			);

			// sleep until the next refresh timer is hit or stop on Ctrl-C
			select! {
				() = sleep(remaining_time) => (),
				() = ctrlc_signaled(self.ctrlc_chan.as_mut()) => {
					tracing::info!("Job {} is shutting down...", self.name);
					return JobResult::Ok;
				}
			}
		}
	}
}

impl<T, H> Job<T, H>
where
	T: TaskGroup,
	H: HandleError,
{
	#[expect(clippy::same_name_method, reason = "can't think of a better name")] // if any come up, I'd be fine to replace it
	pub async fn run(&mut self) -> JobResult {
		// Error handling loop: exit out of it only when the job finishes or a fatal error occures, otherwise run the job once more
		loop {
			// if any errors occured, extract and handle them. Otherwise forward the result(e.g. Ok or Panicked)
			let errors = match self.run_until_first_error().await {
				JobResult::Err(errors) => errors,
				other => return other,
			};

			let cx = HandleErrorContext {
				job_name: &self.name,
				job_refresh_time: self.refresh_time.as_ref(),
				ctrlc_chan: self.ctrlc_chan.as_mut(),
			};

			match self.error_handling.handle_errors(errors, cx).await {
				HandleErrorResult::ContinueJob => (),
				HandleErrorResult::StopAndReturnErrs(e) => return JobResult::Err(e),
				HandleErrorResult::ErrWhileHandling {
					err,
					original_errors,
				} => {
					tracing::error!(
						"An error occured while handling other errors! Stopping the job and returning the original errors.\nDetails: {err}",
					);

					return JobResult::Err(original_errors);
				}
			}
		}
	}
}

impl<T, H> OpaqueJob for Job<T, H>
where
	T: TaskGroup,
	H: HandleError,
{
	async fn run(&mut self) -> JobResult {
		Job::run(self).await
	}

	fn name(&self) -> Option<&str> {
		Some(&self.name)
	}
}
