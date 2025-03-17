/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

pub mod timepoint;

use tokio::{join, time::sleep};

use self::timepoint::TimePoint;
use crate::{
	action::Action, error::FetcherError, external_save::ExternalSave, source::Source, task::Task,
};

/// A single job, containing a single or a couple [`tasks`](`Task`), possibly refetching every set amount of time
#[derive(Debug)]
pub struct Job<T> {
	/// The tasks to run
	pub tasks: T,

	/// Refresh/refetch/redo the job every "this" point of the day
	pub refresh_time: Option<TimePoint>,
}

impl<T> Job<T>
where
	T: RunTasks,
{
	/// Run this job to completion or return early on an error
	///
	/// # Errors
	/// if any of the inner tasks return an error, refer to [`Task`] documentation
	pub async fn run(&mut self) -> Result<(), Vec<FetcherError>> {
		loop {
			let results = self.tasks.run().await;

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

pub trait RunTask {
	async fn run(&mut self) -> Result<(), FetcherError>;
}

pub trait RunTasks {
	async fn run(&mut self) -> Vec<Result<(), FetcherError>>;
}

impl<S, A, E> RunTask for Task<S, A, E>
where
	S: Source,
	A: Action,
	E: ExternalSave + 'static,
{
	async fn run(&mut self) -> Result<(), FetcherError> {
		Task::run(self).await
	}
}

impl RunTask for ! {
	async fn run(&mut self) -> Result<(), FetcherError> {
		unreachable!()
	}
}

impl<T1> RunTasks for T1
where
	T1: RunTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		vec![RunTask::run(self).await]
	}
}

impl<T1> RunTasks for (T1,)
where
	T1: RunTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		vec![self.0.run().await]
	}
}

impl<T1, T2> RunTasks for (T1, T2)
where
	T1: RunTask,
	T2: RunTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		let results = join!(self.0.run(), self.1.run());
		vec![results.0, results.1]
	}
}

impl<T1, T2, T3> RunTasks for (T1, T2, T3)
where
	T1: RunTask,
	T2: RunTask,
	T3: RunTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		let results = join!(self.0.run(), self.1.run(), self.2.run());
		vec![results.0, results.1, results.2]
	}
}

impl<T1, T2, T3, T4> RunTasks for (T1, T2, T3, T4)
where
	T1: RunTask,
	T2: RunTask,
	T3: RunTask,
	T4: RunTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		let results = join!(self.0.run(), self.1.run(), self.2.run(), self.3.run());
		vec![results.0, results.1, results.2, results.3]
	}
}

impl<T1, T2, T3, T4, T5> RunTasks for (T1, T2, T3, T4, T5)
where
	T1: RunTask,
	T2: RunTask,
	T3: RunTask,
	T4: RunTask,
	T5: RunTask,
{
	async fn run(&mut self) -> Vec<Result<(), FetcherError>> {
		let results = join!(
			self.0.run(),
			self.1.run(),
			self.2.run(),
			self.3.run(),
			self.4.run()
		);
		vec![results.0, results.1, results.2, results.3, results.4]
	}
}
