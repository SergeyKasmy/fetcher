/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source::reddit::Reddit as CReddit;
use fetcher_core::source::reddit::Sort as CSort;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Reddit {
	subreddit: String,
	sort: Sort,
	score_threshold: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
#[rustfmt::skip]	// to put new and latest side by side
pub enum Sort {
	Latest, New,
	Rising,
	Hot,
	Top(TimePeriod),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TimePeriod {
	Today,
	ThisWeek,
	ThisMonth,
	ThisYear,
	AllTime,
}

impl Reddit {
	pub fn parse(self) -> CReddit {
		CReddit::new(&self.subreddit, self.sort.parse(), self.score_threshold)
	}
}

impl Sort {
	pub fn parse(self) -> CSort {
		match self {
			Sort::Latest | Sort::New => CSort::Latest,
			Sort::Rising => CSort::Rising,
			Sort::Hot => CSort::Hot,
			Sort::Top(TimePeriod::Today) => CSort::TopDay,
			Sort::Top(TimePeriod::ThisWeek) => CSort::TopWeek,
			Sort::Top(TimePeriod::ThisMonth) => CSort::TopMonth,
			Sort::Top(TimePeriod::ThisYear) => CSort::TopYear,
			Sort::Top(TimePeriod::AllTime) => CSort::TopAllTime,
		}
	}
}