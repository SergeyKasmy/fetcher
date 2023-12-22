/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module defines the [`TimePoint`] enum that specifies either a duration or a time of day a job should be refreshed

use chrono::{offset::Local as LocalTime, NaiveDateTime, NaiveTime};
use std::time::Duration;

/// A point in time of a day
#[derive(Debug)]
pub enum TimePoint {
	/// A duration, always returns itself
	Duration(Duration),

	/// A point in time of a day
	Time(NaiveTime),
}

impl TimePoint {
	/// Returns the duration that is left to the next appropriate point in the day from now
	#[must_use]
	pub fn remaining_from_now(&self) -> Duration {
		let now = LocalTime::now().naive_local();
		self.remaining_from(now)
	}

	/// Returns the duration that is left to the next appropriate point in the day from the provided time `now`
	#[allow(clippy::missing_panics_doc)] // doesn't actually panic
	#[must_use]
	pub fn remaining_from(&self, now: NaiveDateTime) -> Duration {
		match self {
			TimePoint::Duration(dur) => *dur,
			TimePoint::Time(time) => {
				let remaining_time = *time - now.time();

				// return if duration is not negative, i.e. it is in the future.
				// Assumes that [`chrono::Duration::to_std()`] errors otherwise
				match remaining_time.to_std() {
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
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]

	use chrono::NaiveDate;
	use once_cell::sync::Lazy;

	use super::*;

	#[allow(clippy::identity_op)]
	const HOUR: Duration = Duration::from_secs(
		1 /* hour */ * 60 /* mins in hour */ * 60, /* secs in min */
	);

	// assume now is exactly 12 PM
	static NOW: Lazy<NaiveDateTime> = Lazy::new(|| {
		NaiveDateTime::new(
			NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
			NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
		)
	});

	#[test]
	fn duration() {
		let time_point = TimePoint::Duration(HOUR * 5);

		assert_eq!(time_point.remaining_from(*NOW), HOUR * 5);
	}

	#[test]
	fn time_not_yet_passed() {
		let at_2_pm = TimePoint::Time(NaiveTime::from_hms_opt(14, 0, 0).unwrap());

		assert_eq!(at_2_pm.remaining_from(*NOW), HOUR * 2);
	}

	#[test]
	fn time_already_passed() {
		let at_10_am = TimePoint::Time(NaiveTime::from_hms_opt(10, 0, 0).unwrap());

		assert_eq!(at_10_am.remaining_from(*NOW), HOUR * 22);
	}
}
