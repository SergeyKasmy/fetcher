/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module defines the [`Trigger`] trait and its provided implementations [`Never`], [`Every`], and [`OnceADayAt`].
//!
//! It also re-exports [`chrono`] to make the [`NaiveTime`](`chrono::NaiveTime`) easily available.

mod every;
mod never;
mod once_a_day;

pub use self::{every::Every, never::Never, once_a_day::OnceADayAt};

pub use chrono;

use either::Either;

use std::{convert::Infallible, error::Error, fmt::Display, future::Future, time::Duration};

use crate::maybe_send::{MaybeSend, MaybeSendSync};

/// Specifies the condition under which a job will be re-triggered
pub trait Trigger: MaybeSendSync {
	/// The error type that can be returned while waiting
	type Err: Into<Box<dyn Error + Send + Sync>>;

	/// Block the current job until the appropriate condition is met
	///
	/// # Returns
	/// `Ok(TriggerResult::Resume)` if the job should be retriggered after the wait has ended
	/// `Ok(TriggerResult::Stop)` if the job should be stopped after the wait has ended
	/// `Err(Self::Err)` if an error occured while waiting
	fn wait(&mut self) -> impl Future<Output = Result<TriggerResult, Self::Err>> + MaybeSend;

	// TODO: not a very nice API. Provide more freedom for trigger and handleerror implementations to handle errors and waiting as they want.
	// This one is too heavily coupled with ExponentialBackoff specifically
	/// Returns twice of the approximate duration of the typical [`Trigger::wait`].
	///
	/// Currently used as a way to calculate that long enough has passed
	/// and the error counter (if present in the implementation of [`HandleError`](`super::error_handling::HandleError`))
	/// can be reset, and thus it can be assumed that the next error isn't a consecutive error but a new one.
	// TODO: improve docs
	fn twice_as_duration(&self) -> Duration;

	/// Like [`Trigger::wait`] but is called just once when the job starts for the first time.
	///
	/// This function can just delegate itself to [`Trigger::wait`] to wait for the trigger first before running the job.
	/// The default implementation just immediately returns [`TriggerResult::Resume`] to make the job run once before waiting for the trigger.
	fn wait_start(&mut self) -> impl Future<Output = Result<TriggerResult, Self::Err>> + MaybeSend {
		async { Ok(TriggerResult::Resume) }
	}
}

/// What should happen after the [`Trigger::wait`] has ended?
#[derive(Clone, Copy, Debug)]
pub enum TriggerResult {
	/// The job should be resumed and re-triggered
	Resume,

	/// The job should just be stopped
	Stop,
}

/// Forward implementation to [`Never`]
impl Trigger for () {
	type Err = <Never as Trigger>::Err;

	async fn wait(&mut self) -> Result<TriggerResult, Self::Err> {
		Never.wait().await
	}

	fn twice_as_duration(&self) -> Duration {
		Never.twice_as_duration()
	}
}

impl<T: Trigger> Trigger for Option<T> {
	type Err = T::Err;

	async fn wait(&mut self) -> Result<TriggerResult, Self::Err> {
		match self {
			Some(inner) => inner.wait().await,
			None => Never.wait().await.map_err(|e| match e {}),
		}
	}

	fn twice_as_duration(&self) -> Duration {
		match self {
			Some(inner) => inner.twice_as_duration(),
			None => Never.twice_as_duration(),
		}
	}
}

impl<T: Trigger> Trigger for &mut T {
	type Err = T::Err;

	fn wait(&mut self) -> impl Future<Output = Result<TriggerResult, Self::Err>> + MaybeSend {
		(**self).wait()
	}

	fn twice_as_duration(&self) -> Duration {
		(**self).twice_as_duration()
	}
}

impl<A, B> Trigger for Either<A, B>
where
	A: Trigger,
	B: Trigger,
{
	type Err = Box<dyn Error + Send + Sync>;

	async fn wait(&mut self) -> Result<TriggerResult, Self::Err> {
		match self {
			Either::Left(tr) => tr.wait().await.map_err(Into::into),
			Either::Right(tr) => tr.wait().await.map_err(Into::into),
		}
	}

	fn twice_as_duration(&self) -> Duration {
		self.as_ref()
			.map_either(Trigger::twice_as_duration, Trigger::twice_as_duration)
			.into_inner()
	}
}

impl Trigger for Infallible {
	type Err = Infallible;

	async fn wait(&mut self) -> Result<TriggerResult, Self::Err> {
		match *self {}
	}

	fn twice_as_duration(&self) -> Duration {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl Trigger for ! {
	type Err = !;

	async fn wait(&mut self) -> Result<TriggerResult, Self::Err> {
		match *self {}
	}

	fn twice_as_duration(&self) -> Duration {
		match *self {}
	}
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
