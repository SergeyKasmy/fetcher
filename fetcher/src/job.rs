/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

mod job_group;
mod timepoint;

pub use self::{job_group::JobGroup, timepoint::TimePoint};

use std::convert::Infallible;

use tokio::time::sleep;

use crate::{StaticStr, error::FetcherError, task::TaskGroup};

/// A single job, containing a single or a couple [`tasks`](`Task`), possibly refetching every set amount of time
#[derive(bon::Builder, Debug)]
pub struct Job<T> {
	#[builder(start_fn, into)]
	pub name: StaticStr,

	/// The tasks to run
	pub tasks: T,

	/// Refresh/refetch/redo the job every "this" point of the day
	pub refresh_time: Option<TimePoint>,
}

pub trait OpaqueJob {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>>;

	fn disable(self) -> DisabledJob<Self>
	where
		Self: Sized,
	{
		DisabledJob(self)
	}

	async fn make_dry(&mut self) {}
}

impl<T: TaskGroup> OpaqueJob for Job<T> {
	/// Run this job to completion or return early on an error
	///
	/// # Errors
	/// if any of the inner tasks return an error, refer to [`Task`] documentation
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
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

	async fn make_dry(&mut self) {
		self.tasks.make_dry().await;
	}
}

impl OpaqueJob for () {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		Ok(())
	}
}

impl<J> OpaqueJob for Option<J>
where
	J: OpaqueJob,
{
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		let Some(job) = self else {
			return Ok(());
		};

		job.run().await
	}

	async fn make_dry(&mut self) {
		if let Some(job) = self {
			job.make_dry().await;
		}
	}
}

pub struct DisabledJob<J>(J);

impl<J> OpaqueJob for DisabledJob<J> {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		Ok(())
	}
}

impl OpaqueJob for Infallible {
	async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		unreachable!()
	}
}
