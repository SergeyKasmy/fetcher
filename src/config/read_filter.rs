/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::read_filter;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReadFilterKind {
	NewerThanRead,
	NotPresentInReadList,
}

impl ReadFilterKind {
	pub(crate) fn parse(self) -> read_filter::ReadFilterKind {
		match self {
			ReadFilterKind::NewerThanRead => read_filter::ReadFilterKind::NewerThanLastRead,
			ReadFilterKind::NotPresentInReadList => {
				read_filter::ReadFilterKind::NotPresentInReadList
			}
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum ReadFilter {
	NewerThanRead(ReadFilterNewer),
	NotPresentInReadList(ReadFilterNotPresent),
}

impl ReadFilter {
	pub(crate) fn parse(self, name: &str) -> read_filter::ReadFilter {
		let inner = match self {
			ReadFilter::NewerThanRead(x) => {
				read_filter::ReadFilterInner::NewerThanLastRead(x.parse())
			}
			ReadFilter::NotPresentInReadList(x) => {
				read_filter::ReadFilterInner::NotPresentInReadList(x.parse())
			}
		};

		read_filter::ReadFilter {
			name: name.to_owned(),
			inner,
		}
	}

	pub(crate) fn unparse(read_filter: &read_filter::ReadFilterInner) -> Option<Self> {
		Some(match read_filter {
			read_filter::ReadFilterInner::NewerThanLastRead(x) => {
				ReadFilter::NewerThanRead(ReadFilterNewer::unparse(x)?)
			}
			read_filter::ReadFilterInner::NotPresentInReadList(x) => {
				ReadFilter::NotPresentInReadList(ReadFilterNotPresent::unparse(x)?)
			}
		})
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ReadFilterNewer {
	last_read_id: String,
}

impl ReadFilterNewer {
	pub(crate) fn parse(self) -> read_filter::newer::ReadFilterNewer {
		read_filter::newer::ReadFilterNewer {
			last_read_id: Some(self.last_read_id),
		}
	}

	pub(crate) fn unparse(read_filter: &read_filter::newer::ReadFilterNewer) -> Option<Self> {
		read_filter.last_read_id.as_ref().map(|last_read_id| Self {
			last_read_id: last_read_id.to_owned(),
		})
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ReadFilterNotPresent {
	read_list: Vec<String>,
}

impl ReadFilterNotPresent {
	pub(crate) fn parse(self) -> read_filter::not_present::ReadFilterNotPresent {
		read_filter::not_present::ReadFilterNotPresent {
			read_list: self.read_list.into(),
		}
	}

	pub(crate) fn unparse(
		read_filter: &read_filter::not_present::ReadFilterNotPresent,
	) -> Option<Self> {
		if !read_filter.read_list.is_empty() {
			Some(Self {
				read_list: read_filter.read_list.iter().map(|s| s.clone()).collect(),
			})
		} else {
			None
		}
	}
}
