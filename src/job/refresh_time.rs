/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module defines the [`RefreshTime`] type
//! that specifies either a duration or a time of day a job should be refreshed.
//!
//! It also re-exported [`chrono`] to make use of [`NaiveTime`] and [`NaiveDateTime`] types.

pub use chrono;

use chrono::{NaiveDateTime, NaiveTime, offset::Local as LocalTime};
use std::time::Duration;

/// When to refresh the job?
#[derive(Clone, Debug)]
pub enum RefreshTime {
	/// After this mount of time has passed since the last time
	Every(Duration),

	/// Once a day at this time
	OnceADayAt(NaiveTime),

	/// Never refresh, run once
	Never,
}

impl RefreshTime {
	/// Returns the duration that is left to the next time a refresh should be run, from now
	#[must_use]
	pub fn remaining_time_from_now(&self) -> Option<Duration> {
		let now = LocalTime::now().naive_local();
		self.remaining_time_from(now)
	}

	/// Returns the duration that is left to the next time a refresh should be run, from the provided time `now`
	#[expect(
		clippy::missing_panics_doc,
		reason = "doesn't actually panic, unless bugged"
	)]
	#[must_use]
	pub fn remaining_time_from(&self, now: NaiveDateTime) -> Option<Duration> {
		match self {
			RefreshTime::Every(dur) => Some(*dur),
			RefreshTime::OnceADayAt(time) => {
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
			RefreshTime::Never => None,
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
		assert_eq!(RefreshTime::Never.remaining_time_from_now(), None);
	}

	#[test]
	fn every() {
		let time_point = RefreshTime::Every(HOUR * 5);

		assert_eq!(time_point.remaining_time_from(*NOW).unwrap(), HOUR * 5);
	}

	#[test]
	fn once_a_day_today() {
		let at_2_pm = RefreshTime::OnceADayAt(NaiveTime::from_hms_opt(14, 0, 0).unwrap());

		assert_eq!(at_2_pm.remaining_time_from(*NOW).unwrap(), HOUR * 2);
	}

	#[test]
	fn once_a_day_tomorrow() {
		let at_10_am = RefreshTime::OnceADayAt(NaiveTime::from_hms_opt(10, 0, 0).unwrap());

		assert_eq!(at_10_am.remaining_time_from(*NOW).unwrap(), HOUR * 22);
	}
}
