/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

pub mod error_handling;
pub mod job_group;
pub mod opaque_job;
pub mod refresh_time;

mod job_result;

pub use self::{
	error_handling::HandleError, job_group::JobGroup, job_result::JobResult, opaque_job::OpaqueJob,
	refresh_time::RefreshTime,
};

use error_handling::{HandleErrorContext, HandleErrorResult};
use futures::FutureExt;
use std::panic;
use tokio::{select, time::sleep};

use crate::{
	StaticStr,
	ctrl_c_signal::{CtrlCSignalChannel, ctrlc_wait},
	error::ErrorChainDisplay,
	task::TaskGroup,
};

/// A single job, containing a single or a couple [`tasks`](`crate::task::Task`), possibly refetching every set amount of time
#[derive(bon::Builder, Debug)]
#[builder(finish_fn(name = "build_internal", vis = ""))]
#[builder(builder_type(doc {
/// Use builder syntax to set the inputs and finish with [`build()`](`JobBuilder::build()`)
/// or [`build_with_default_error_handling()`](`JobBuilder::build_with_default_error_handling()`).
}))]
#[non_exhaustive]
pub struct Job<T, H> {
	/// Name of the job
	#[builder(start_fn, into)]
	pub name: StaticStr,

	/// Tasks/pipeline to run the data through
	pub tasks: T,

	/// Refresh/refetch/redo the job every provided amount of time
	pub refresh_time: RefreshTime,

	/// Handler for errors that occur during job execution
	pub error_handling: H,

	/// Gracefully stop the job when a Ctrl-C signal arrives
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

			let Some(remaining_time) = self.refresh_time.remaining_time_from_now() else {
				return JobResult::Ok;
			};

			tracing::debug!(
				"Putting job to sleep for {}m",
				remaining_time.as_secs() / 60
			);

			// sleep until the next refresh timer is hit or stop on Ctrl-C
			select! {
				() = sleep(remaining_time) => (),
				() = ctrlc_wait(self.ctrlc_chan.as_mut()) => {
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
	/// Runs the job until it finishes (which can only occur without a [`Job::refresh_time`]),
	/// or until an error or a panic happens.
	///
	/// # Note
	/// This function never panics. If a panic occures, [`JobResult::Panicked`] is just returned instead.
	#[expect(clippy::same_name_method, reason = "can't think of a better name")] // if any come up, I'd be fine to replace it
	pub async fn run(&mut self) -> JobResult {
		match panic::AssertUnwindSafe(self.run_inner())
			.catch_unwind()
			.await
		{
			Ok(job_result) => job_result,
			Err(panic_payload) => JobResult::Panicked {
				payload: panic_payload,
			},
		}
	}

	async fn run_inner(&mut self) -> JobResult {
		// Error handling loop: exit out of it only when the job finishes or a fatal error occures, otherwise run the job once more
		loop {
			// if any errors occured, extract and handle them. Otherwise forward the result(e.g. Ok or Panicked)
			let errors = match self.run_until_first_error().await {
				JobResult::Err(errors) => errors,
				other => return other,
			};

			let cx = HandleErrorContext {
				job_name: &self.name,
				job_refresh_time: &self.refresh_time,
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

impl<T, S: job_builder::State> JobBuilder<T, error_handling::ExponentialBackoff, S>
where
	T: TaskGroup,
{
	/// Finish building and return the requested object
	/// with default error handling ([`ExponentialBackoff`](`error_handling::ExponentialBackoff`)).
	///
	/// # Note
	/// `T` is constrained to implement [`TaskGroup`]
	/// because the builder propagates the [`CtrlCSignalChannel`]
	/// too all child tasks on build.
	pub fn build_with_default_error_handling(self) -> Job<T, error_handling::ExponentialBackoff>
	where
		S: job_builder::IsComplete,
		S::CtrlcChan: job_builder::IsSet,
		S::ErrorHandling: job_builder::IsUnset,
		S::RefreshTime: job_builder::IsSet,
		S::Tasks: job_builder::IsSet,
	{
		let this = self.error_handling(error_handling::ExponentialBackoff::default());
		this.build()
	}
}

impl<T, H, S: job_builder::State> JobBuilder<T, H, S>
where
	T: TaskGroup,
{
	/// Finish building and return the requested object.
	///
	/// # Note
	/// `T` is constrained to implement [`TaskGroup`]
	/// because the builder propagates the [`CtrlCSignalChannel`]
	/// too all child tasks on build.
	pub fn build(self) -> Job<T, H>
	where
		S: job_builder::IsComplete,
		S::CtrlcChan: job_builder::IsSet,
		S::ErrorHandling: job_builder::IsSet,
		S::RefreshTime: job_builder::IsSet,
		S::Tasks: job_builder::IsSet,
	{
		let mut job = self.build_internal();

		if let Some(ctrlc_chan) = &job.ctrlc_chan {
			job.tasks.set_ctrlc_channel(ctrlc_chan.clone());
		}

		job
	}
}
