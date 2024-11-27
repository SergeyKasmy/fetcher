/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source::{Reddit as CReddit, reddit::Sort as CSort};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Reddit(pub HashMap<String, Inner>);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Inner {
	sort: Sort,
	score_threshold: Option<u32>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
#[rustfmt::skip]	// to put new and latest side by side
pub enum Sort {
	Latest, New,
	Rising,
	Hot,
	Top(TimePeriod),
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TimePeriod {
	Today,
	ThisWeek,
	ThisMonth,
	ThisYear,
	AllTime,
}

impl Reddit {
	#[must_use]
	pub fn decode_from_conf(self) -> Vec<CReddit> {
		self.0
			.into_iter()
			.map(|(subreddit, inner)| inner.decode_from_conf(&subreddit))
			.collect()
	}
}

impl Inner {
	#[must_use]
	pub fn decode_from_conf(self, subreddit: &str) -> CReddit {
		CReddit::new(
			subreddit,
			self.sort.decode_from_conf(),
			self.score_threshold,
		)
	}
}

impl Sort {
	#[must_use]
	pub fn decode_from_conf(self) -> CSort {
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
