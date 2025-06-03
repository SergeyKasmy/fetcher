/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

pub mod error_handling;
pub mod job_group;
pub mod opaque_job;
mod simple_job;
pub mod trigger;

mod job_result;

pub use self::{
	error_handling::HandleError, job_group::JobGroup, job_result::JobResult, opaque_job::OpaqueJob,
	trigger::Trigger,
};

use futures::FutureExt;
use non_non_full::NonEmptyVec;
use simple_job::{SimpleJob, SimpleJobBuilder};
use std::ops::ControlFlow;
use std::panic;
use tokio::select;

use self::error_handling::{HandleErrorContext, HandleErrorResult};
use self::trigger::TriggerResult;
use crate::{
	StaticStr,
	cancellation_token::{CancellationToken, cancel_wait},
	error::ErrorChainDisplay,
	maybe_send::MaybeSync,
	task::TaskGroup,
};

// TODO: create a "small job" that combines job and task builders into one for small tasks
/// A single job, containing a single or a couple [`tasks`](`crate::task::Task`), possibly refetching every set amount of time
#[derive(bon::Builder, Clone, Debug)]
#[builder(finish_fn(name = "build_internal", vis = ""))]
#[builder(builder_type(doc {
/// Use builder syntax to set the inputs and finish with [`build()`](`JobBuilder::build()`)
/// or [`build_with_default_error_handling()`](`JobBuilder::build_with_default_error_handling()`).
}))]
#[non_exhaustive]
pub struct Job<T, Tr, H> {
	/// Name of the job
	#[builder(start_fn, into)]
	pub name: StaticStr,

	/// Tasks/pipeline to run the data through
	pub tasks: T,

	/// Trigger the job at the provided intervals or when the trigger condition is met
	pub trigger: Tr,

	/// Handler for errors that occur during job execution
	pub error_handling: H,

	/// Gracefully stop the job when signalled
	#[builder(required)]
	pub cancel_token: Option<CancellationToken>,
}

impl<Tr, H> Job<(), Tr, H> {
	/// Creates an instance of [`Job`] using the builder syntax
	/// that providers setters for all fields of a job with 1 child task
	///
	/// This builder is specialized for jobs containing just a single task
	/// and makes it less bolierplate-y to create simpler jobs.
	///
	/// # Example
	#[cfg_attr(all(feature = "source-http", feature = "action-html"), doc = "```")]
	#[cfg_attr(
		not(all(feature = "source-http", feature = "action-html")),
		doc = " ```ignore"
	)]
	/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
	/// use fetcher::{
	///     Task,
	///     Job,
	///     actions::{sink, transform, transforms::Html},
	///     sinks::Stdout,
	///     sources::{Fetch, Http},
	///     job::trigger,
	/// };
	/// use std::time::Duration;
	///
	/// let task = Task::builder("example")
	///     .source(Http::new_get("https://ipinfo.io/ip")?.into_source_without_read_filter())
	///     .tag("example_tag")
	///     .action((
	///         transform(Html::builder().text("body > pre").unwrap().build()),
	///         sink(Stdout),
	///     ))
	///     .build_without_replies();
	///
	/// // these 2 jobs are the same
	/// let _job = Job::builder("example")
	///     .tasks(task)
	///     .trigger(trigger::Every(Duration::from_secs(1)))
	///     .cancel_token(None)
	///     .build_with_default_error_handling();
	///
	/// let _simple_job = Job::builder_simple("example")
	///     .source(Http::new_get("https://ipinfo.io/ip")?.into_source_without_read_filter())
	///     .tag("example_tag")
	///     .action((
	///         transform(Html::builder().text("body > pre").unwrap().build()),
	///         sink(Stdout),
	///     ))
	///     .trigger(trigger::Every(Duration::from_secs(1)))
	///     .cancel_token(None)
	///     .build_with_default_error_handling();
	/// # Ok(())
	/// # }
	#[doc = "```"]
	pub fn builder_simple<S, A>(name: impl Into<StaticStr>) -> SimpleJobBuilder<S, A, Tr, H> {
		SimpleJob::builder(name)
	}
}

impl<T, Tr, H> Job<T, Tr, H>
where
	T: TaskGroup,
	Tr: Trigger,
{
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

		match self.trigger.wait_start().await {
			Ok(TriggerResult::Resume) => (),
			Ok(TriggerResult::Stop) => return JobResult::Ok,
			Err(e) => return JobResult::TriggerFailed(e.into()),
		}

		// Job loop: break out of it only on errors or if the job runs only once
		loop {
			let results = self.tasks.run_concurrently().await;

			#[expect(clippy::manual_ok_err, reason = "false positive")]
			let errors = results
				.into_iter()
				.filter_map(|r| {
					tracing::trace!("Task result: {r:?}");

					match r {
						Ok(()) => None,
						Err(e) => Some(e),
					}
				})
				.collect::<Vec<_>>();

			// returns errors if any
			if let Some(errors) = NonEmptyVec::new(errors) {
				return JobResult::Err(errors);
			}

			match wait_for_trigger(&mut self.trigger, self.cancel_token.as_mut(), &self.name).await
			{
				ControlFlow::Continue(()) => (),
				ControlFlow::Break(res) => return res,
			}
		}
	}
}

impl<T, Tr, H> Job<T, Tr, H>
where
	T: TaskGroup,
	Tr: Trigger + MaybeSync,
	H: HandleError<Tr>,
{
	/// Runs the job until it finishes (which can only occur without a [`Job::trigger`]),
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

	// TODO: move it inside the main run loop and handle errors in a match arm? This would avoid the duplicated wait_for_trigger call and allow it to be inlined
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
				job_trigger: &self.trigger,
				cancel_token: self.cancel_token.as_mut(),
			};

			match self.error_handling.handle_errors(errors, cx).await {
				HandleErrorResult::ResumeJob => (),
				HandleErrorResult::StopWithErrors(e) => return JobResult::Err(e),
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

			match wait_for_trigger(&mut self.trigger, self.cancel_token.as_mut(), &self.name).await
			{
				ControlFlow::Continue(()) => (),
				ControlFlow::Break(res) => return res,
			}
		}
	}
}

impl<T, Tr, H> OpaqueJob for Job<T, Tr, H>
where
	T: TaskGroup,
	Tr: Trigger + MaybeSync,
	H: HandleError<Tr>,
{
	async fn run(&mut self) -> JobResult {
		Job::run(self).await
	}

	fn name(&self) -> Option<&str> {
		Some(&self.name)
	}
}

impl<T, Tr, S: job_builder::State> JobBuilder<T, Tr, error_handling::ExponentialBackoff, S>
where
	T: TaskGroup,
{
	/// Finish building and return the requested object
	/// with default error handling ([`ExponentialBackoff`](`error_handling::ExponentialBackoff`)).
	///
	/// # Note
	/// `T` is constrained to implement [`TaskGroup`]
	/// because the builder propagates the [`CancellationToken`]
	/// too all child tasks on build.
	pub fn build_with_default_error_handling(self) -> Job<T, Tr, error_handling::ExponentialBackoff>
	where
		S::CancelToken: job_builder::IsSet,
		S::ErrorHandling: job_builder::IsUnset,
		S::Trigger: job_builder::IsSet,
		S::Tasks: job_builder::IsSet,
	{
		let this = self.error_handling(error_handling::ExponentialBackoff::default());
		this.build()
	}
}

impl<T, Tr, H, S: job_builder::State> JobBuilder<T, Tr, H, S>
where
	T: TaskGroup,
{
	/// Finish building and return the requested object.
	///
	/// # Note
	/// `T` is constrained to implement [`TaskGroup`]
	/// because the builder propagates the [`CancellationToken`]
	/// too all child tasks on build.
	pub fn build(self) -> Job<T, Tr, H>
	where
		S: job_builder::IsComplete,
		S::CancelToken: job_builder::IsSet,
		S::ErrorHandling: job_builder::IsSet,
		S::Trigger: job_builder::IsSet,
		S::Tasks: job_builder::IsSet,
	{
		let mut job = self.build_internal();

		if let Some(token) = &job.cancel_token {
			job.tasks.set_cancel_token(token.clone());
		}

		job
	}
}

/// Sleep until the next trigger is hit or stop when the cancel token is triggered
///
/// # Returns
/// `ControlFlow::Continue(())` if the job should be resumed
/// `ControlFlow::Break(res)` if the job should stop and return `res`
async fn wait_for_trigger<Tr>(
	mut trigger: Tr,
	cancel_token: Option<&mut CancellationToken>,
	job_name: &str,
) -> ControlFlow<JobResult>
where
	Tr: Trigger,
{
	select! {
		trigger_res = trigger.wait() => {
			match trigger_res {
				Ok(TriggerResult::Resume) => ControlFlow::Continue(()),
				Ok(TriggerResult::Stop) => ControlFlow::Break(JobResult::Ok),
				Err(e) => ControlFlow::Break(JobResult::TriggerFailed(e.into())),
			}
		},
		() = cancel_wait(cancel_token) => {
			tracing::info!("Job {job_name} is shutting down...");
			ControlFlow::Break(JobResult::Ok)
		}
	}
}
