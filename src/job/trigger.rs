/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module defines the [`Trigger`] type
//! that specifies either a duration or a time of day a job should be re-triggered.
//!
//! It also re-exported [`chrono`] to make use of [`NaiveTime`] and [`NaiveDateTime`] types.

mod every;
mod never;
mod once_a_day;

pub use self::{every::Every, never::Never, once_a_day::OnceADayAt};

pub use chrono;

use std::{error::Error, fmt::Display, time::Duration};

use crate::maybe_send::{MaybeSend, MaybeSendSync};

/// Specifies the condition under which a job will be re-triggered
pub trait Trigger: MaybeSendSync {
	/// The error type that can be returned while waiting
	type Err: Into<Box<dyn Error + Send + Sync>>;

	/// Block the current job until the appropriate condition is met
	///
	/// # Returns
	/// `Ok(ContinueJob::Yes)` if the job should be retriggered after the wait has ended
	/// `Ok(ContinueJob::No)` if the job should be stopped after the wait has ended
	/// `Err(Self::Err)` if an error occured while waiting
	fn wait(&mut self) -> impl Future<Output = Result<ContinueJob, Self::Err>> + MaybeSend;

	// TODO: not a very nice API. Provide more freedom for trigger and handleerror implementations to handle errors and waiting as they want.
	// This one is too heavily coupled with ExponentialBackoff specifically
	/// Returns twice of the approximate duration of the typical [`Trigger::wait`].
	///
	/// Currently used as a way to calculate that long enough has passed
	/// and the error counter (if present in the implementation of [`HandleErrors`](`super::error_handling::HandleErrors`))
	/// can be reset, and thus it can be assumed that the next error isn't a consecutive error but a new one.
	// TODO: improve docs
	fn twice_as_duration(&self) -> Duration;
}

/// What should happen after the [`Trigger::wait`] has ended?
#[derive(Clone, Copy, Debug)]
pub enum ContinueJob {
	/// The job should be continued and re-triggered
	Yes,

	/// The job should just be stopped
	No,
}

async fn sleep(duration: Duration) {
	const SECS_IN_MIN: u64 = 60;

	// log remaining sleep time
	// scope to avoid keeping &dyn Display's across .await points
	{
		let mins = duration.as_secs() / SECS_IN_MIN;
		let remainder = duration.as_secs() % SECS_IN_MIN;
		let show_remaining_secs = mins < 5 && remainder > 0;
		let display_remainder: (&dyn Display, &'static str) = if show_remaining_secs {
			(&remainder, "s")
		} else {
			(&"", "")
		};

		tracing::debug!(
			"Putting job to sleep for {mins}m{}{}",
			display_remainder.0,
			display_remainder.1
		);
	}

	tokio::time::sleep(duration).await;
}
