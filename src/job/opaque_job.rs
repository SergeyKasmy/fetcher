/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`OpaqueJob`] trait

use std::convert::Infallible;

use crate::maybe_send::{MaybeSend, MaybeSendSync};

use super::JobResult;

/// A trait representing a runnable job.
///
/// This trait is mostly used to type-erase generics off of a [`Job`](`super::Job`)
/// but it can very well be used to implement a job-like interface for a different type entirely.
///
/// This trait provides a minimal interface for types that represent executable jobs,
/// abstracting away the specific details of what the job does. It is designed to be
/// implementation-agnostic, allowing for different types of jobs to be treated uniformly.
///
/// # Default Implementations
///
/// The trait provides default implementations for:
/// - Unit type `()`: A no-op job that always succeeds
/// - `Option<J>`: Runs the inner job if `Some`, returns success if `None`
/// - `Infallible`: A job that can never be constructed or run
pub trait OpaqueJob: MaybeSendSync {
	/// Executes the job.
	///
	/// This method is the core of the job's functionality. When called, it should perform
	/// the job's work and return a [`JobResult`] indicating success or failure.
	fn run(&mut self) -> impl Future<Output = JobResult> + MaybeSend;

	/// Returns an optional human-readable name for the job.
	/// It's useful for logging, debugging, and monitoring purposes.
	///
	/// The usual [`Job`](`super::Job`) type always has a name attached to it
	/// but this method returns [`Option`] to allow other implementers to avoid having to provide a name
	/// (as well as to ease implementation of this trait for "empty" or "option"-like types).
	fn name(&self) -> Option<&str> {
		None
	}
}

impl OpaqueJob for () {
	async fn run(&mut self) -> JobResult {
		JobResult::Ok
	}
}

impl<J> OpaqueJob for Option<J>
where
	J: OpaqueJob,
{
	async fn run(&mut self) -> JobResult {
		let Some(job) = self else {
			return JobResult::Ok;
		};

		job.run().await
	}

	fn name(&self) -> Option<&str> {
		self.as_ref().and_then(|x| x.name())
	}
}

impl OpaqueJob for Infallible {
	async fn run(&mut self) -> JobResult {
		unreachable!()
	}
}
