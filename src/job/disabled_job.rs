/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`DisabledJob`] type

use super::OpaqueJob;
use crate::job::JobResult;

/// A wrapper type that disables the execution of a job.
///
/// This type wraps a job and implements [`OpaqueJob`] in a way that
/// makes [`OpaqueJob::run`] a no-op.
///
/// See [`OpaqueJob::disable`].
pub struct DisabledJob<T>(pub T);

impl<T> DisabledJob<T> {
	/// Gets the wrapped job out of [`DisabledJob`].
	///
	/// Pattern matching can be used as well.
	pub fn into_inner(self) -> T {
		self.0
	}
}

impl<T: OpaqueJob> OpaqueJob for DisabledJob<T> {
	async fn run(&mut self) -> JobResult {
		JobResult::Ok
	}

	fn name(&self) -> Option<&str> {
		self.0.name()
	}
}
