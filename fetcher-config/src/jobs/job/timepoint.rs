/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::FetcherConfigError;
use fetcher_core::job::timepoint::TimePoint as CTimePoint;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TimePoint {
	Every(String),
	At(String),
}

impl TimePoint {
	pub fn parse(self) -> Result<CTimePoint, FetcherConfigError> {
		Ok(match self {
			TimePoint::Every(every) => CTimePoint::Duration(duration_str::parse_std(every)?),
			TimePoint::At(at) => {
				let time = chrono::NaiveTime::parse_from_str(&at, "%H:%M")?;
				CTimePoint::Time(time)
			}
		})
	}
}
