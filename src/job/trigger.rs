/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module defines the [`Trigger`] type
//! that specifies either a duration or a time of day a job should be re-triggered.
//!
//! It also re-exported [`chrono`] to make use of [`NaiveTime`] and [`NaiveDateTime`] types.

pub use chrono;

use chrono::{NaiveTime, offset::Local as LocalTime};
use std::{fmt::Display, time::Duration};

use crate::maybe_send::{MaybeSend, MaybeSendSync};

// TODO: add error type
pub trait Trigger: MaybeSendSync {
	fn wait(&mut self) -> impl Future<Output = ContinueJob> + MaybeSend;

	// TODO: not a very nice API. Provide more freedom for trigger and handleerror implementations to handle errors and waiting as they want.
	// This one is too heavily coupled with ExponentialBackoff specifically
	fn twice_as_duration(&self) -> Duration;
}

#[derive(Clone, Copy, Debug)]
pub enum ContinueJob {
	Yes,
	No,
}

#[derive(Clone, Copy, Debug)]
pub struct Every(pub Duration);

impl Trigger for Every {
	async fn wait(&mut self) -> ContinueJob {
		sleep(self.0).await;
		ContinueJob::Yes
	}

	fn twice_as_duration(&self) -> Duration {
		self.0 * 2
	}
}

#[derive(Clone, Copy, Debug)]
pub struct OnceADayAt(pub NaiveTime);

impl Trigger for OnceADayAt {
	async fn wait(&mut self) -> ContinueJob {
		let remaining_time = self.0 - LocalTime::now().naive_local().time();

		let time_left = match remaining_time.to_std() {
			// duration is positive, points to a moment in the future
			Ok(dur) => dur,

			// duration is negative, points to a moment in the past.
			// This means we should add a day and return that
			// since that time today has already passed
			Err(_) => (remaining_time + chrono::Duration::days(1))
				.to_std()
				.expect("should point to the future since we added a day to the current day"),
		};

		sleep(time_left).await;
		ContinueJob::Yes
	}

	fn twice_as_duration(&self) -> Duration {
		const TWO_DAYS: Duration = Duration::from_secs(
			2 /* days */ * 24 /* hours a day */ * 60 /* mins an hour */ * 60, /* secs a min */
		);

		TWO_DAYS
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Never;

impl Trigger for Never {
	async fn wait(&mut self) -> ContinueJob {
		ContinueJob::No
	}

	fn twice_as_duration(&self) -> Duration {
		Duration::ZERO
	}
}

async fn sleep(duration: Duration) {
	const SECS_IN_MIN: u64 = 60;

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

/*
/// When to re-trigger the job?
#[derive(Clone, Debug)]
pub enum Trigger {
	/// After this mount of time has passed since the last time
	Every(Duration),

	/// Once a day at this time
	OnceADayAt(NaiveTime),

	/// Never re-trigger, run once
	Never,
}

impl Trigger {
	/// Returns the duration that is left to the next time the job should be re-triggered, from now
	#[must_use]
	pub fn remaining_time_from_now(&self) -> Option<Duration> {
		let now = LocalTime::now().naive_local();
		self.remaining_time_from(now)
	}

	/// Returns the duration that is left to the next time the job should be re-triggered, from the provided time `now`
	#[expect(
		clippy::missing_panics_doc,
		reason = "doesn't actually panic, unless bugged"
	)]
	#[must_use]
	pub fn remaining_time_from(&self, now: NaiveDateTime) -> Option<Duration> {
		match self {
			Trigger::Every(dur) => Some(*dur),
			Trigger::OnceADayAt(time) => {
				let remaining_time = *time - now.time();

				// return if duration is not negative, i.e. it is in the future.
				// Assumes that [`chrono::Duration::to_std()`] errors otherwise
				let time_left = match remaining_time.to_std() {
					// duration is positive, points to a moment in the future
					Ok(dur) => dur,

					// duration is negative, points to a moment in the past.
					// This means we should add a day and return that
					// since that time today has already passed
					Err(_) => (remaining_time + chrono::Duration::days(1))
						.to_std()
						.expect(
							"should point to the future since we added a day to the current day",
						),
				};

				Some(time_left)
			}
			Trigger::Never => None,
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]

	use std::sync::LazyLock;

	use chrono::NaiveDate;

	use super::*;

	const HOUR: Duration = Duration::from_secs(60 /* mins in hour */ * 60 /* secs in min */);

	// assume now is exactly 12 PM
	static NOW: LazyLock<NaiveDateTime> = LazyLock::new(|| {
		NaiveDateTime::new(
			NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
			NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
		)
	});

	#[test]
	fn never() {
		assert_eq!(Trigger::Never.remaining_time_from_now(), None);
	}

	#[test]
	fn every() {
		let time_point = Trigger::Every(HOUR * 5);

		assert_eq!(time_point.remaining_time_from(*NOW).unwrap(), HOUR * 5);
	}

	#[test]
	fn once_a_day_today() {
		let at_2_pm = Trigger::OnceADayAt(NaiveTime::from_hms_opt(14, 0, 0).unwrap());

		assert_eq!(at_2_pm.remaining_time_from(*NOW).unwrap(), HOUR * 2);
	}

	#[test]
	fn once_a_day_tomorrow() {
		let at_10_am = Trigger::OnceADayAt(NaiveTime::from_hms_opt(10, 0, 0).unwrap());

		assert_eq!(at_10_am.remaining_time_from(*NOW).unwrap(), HOUR * 22);
	}
}
*/
