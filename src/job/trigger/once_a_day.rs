/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{convert::Infallible, time::Duration};

use chrono::{NaiveDateTime, NaiveTime, offset::Local as LocalTime};

use super::{ContinueJob, Trigger, sleep};

/// Re-trigger the job every day once a day at the provided wall clock time
#[derive(Clone, Copy, Debug)]
pub struct OnceADayAt(pub NaiveTime);

impl Trigger for OnceADayAt {
	type Err = Infallible;

	async fn wait(&mut self) -> Result<ContinueJob, Self::Err> {
		let now = LocalTime::now().naive_local();
		let time_remaining = self.time_remaining_from(now);

		sleep(time_remaining).await;
		Ok(ContinueJob::Yes)
	}

	fn twice_as_duration(&self) -> Duration {
		const TWO_DAYS: Duration = Duration::from_secs(
			2 /* days */ * 24 /* hours a day */ * 60 /* mins an hour */ * 60, /* secs a min */
		);

		TWO_DAYS
	}
}

impl OnceADayAt {
	fn time_remaining_from(self, now: NaiveDateTime) -> Duration {
		let time_remaining = self.0 - now.time();

		match time_remaining.to_std() {
			// duration is positive, points to a moment in the future
			Ok(dur) => dur,

			// duration is negative, points to a moment in the past.
			// This means we should add a day and return that
			// since that time today has already passed
			Err(_) => (time_remaining + chrono::Duration::days(1))
				.to_std()
				.expect("should point to the future since we added a day to the current day"),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]

	use std::sync::LazyLock;

	use chrono::{NaiveDate, NaiveDateTime};

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
	fn once_a_day_today() {
		let at_2_pm = OnceADayAt(NaiveTime::from_hms_opt(14, 0, 0).unwrap());

		assert_eq!(at_2_pm.time_remaining_from(*NOW), HOUR * 2);
	}

	#[test]
	fn once_a_day_tomorrow() {
		let at_10_am = OnceADayAt(NaiveTime::from_hms_opt(10, 0, 0).unwrap());

		assert_eq!(at_10_am.time_remaining_from(*NOW), HOUR * 22);
	}
}
