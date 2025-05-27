/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`OpaqueTask`] trait

use super::DisabledTask;
use crate::{
	ctrl_c_signal::CtrlCSignalChannel,
	error::FetcherError,
	maybe_send::{MaybeSend, MaybeSendSync},
};

/// A trait representing a runnable task.
///
/// This trait is mostly used to type-erase generics off of a [`Task`](`super::Task`)
/// but it can very well be used to implement a task-like interface for a different type entirely.
///
/// This trait provides a minimal interface for types that represent executable tasks,
/// abstracting away the specific details of what the task does. It is designed to be
/// implementation-agnostic, allowing for different types of jobs to be treated uniformly.
///
/// # Default Implementations
///
/// The trait provides default implementations for:
/// - Unit type `()`: A no-op task that always succeeds
/// - `Option<T>`: Runs the inner task if `Some`, returns success if `None`
/// - `Infallible`: A job that can never be constructed or run
pub trait OpaqueTask: MaybeSendSync {
	/// Executes the task.
	///
	/// This method is the core of the task's functionality. When called, it should perform
	/// the task's work and return a [`Result`] indicating success or failure.
	fn run(&mut self) -> impl Future<Output = Result<(), FetcherError>> + MaybeSend;

	/// Sets the [`CtrlCChannel`] of the task to `channel`
	fn set_ctrlc_channel(&mut self, channel: CtrlCSignalChannel);

	/// Disables the task, Making [`OpaqueTask::run`] a no-op.
	///
	/// Useful for quicky disabling a task without removing its code.
	fn disable(self) -> DisabledTask<Self>
	where
		Self: Sized,
	{
		DisabledTask(self)
	}
}

impl OpaqueTask for () {
	async fn run(&mut self) -> Result<(), FetcherError> {
		Ok(())
	}

	fn set_ctrlc_channel(&mut self, _channel: CtrlCSignalChannel) {}
}

impl<T> OpaqueTask for Option<T>
where
	T: OpaqueTask,
{
	async fn run(&mut self) -> Result<(), FetcherError> {
		let Some(task) = self else {
			return Ok(());
		};

		task.run().await
	}

	fn set_ctrlc_channel(&mut self, channel: CtrlCSignalChannel) {
		let Some(task) = self else {
			return;
		};

		task.set_ctrlc_channel(channel);
	}
}
