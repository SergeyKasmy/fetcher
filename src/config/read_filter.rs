/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::read_filter;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Kind {
	NewerThanRead,
	NotPresentInReadList,
}

impl Kind {
	pub(crate) fn parse(self) -> read_filter::Kind {
		match self {
			Kind::NewerThanRead => read_filter::Kind::NewerThanLastRead,
			Kind::NotPresentInReadList => read_filter::Kind::NotPresentInReadList,
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReadFilter {
	NewerThanRead(Newer),
	NotPresentInReadList(NotPresent),
}

impl ReadFilter {
	#[must_use]
	pub fn parse(self, external_save: read_filter::Writer) -> read_filter::ReadFilter {
		let inner = match self {
			ReadFilter::NewerThanRead(x) => {
				read_filter::ReadFilterInner::NewerThanLastRead(x.parse())
			}
			ReadFilter::NotPresentInReadList(x) => {
				read_filter::ReadFilterInner::NotPresentInReadList(x.parse())
			}
		};

		read_filter::ReadFilter {
			inner,
			external_save,
		}
	}

	pub(crate) fn unparse(read_filter: &read_filter::ReadFilter) -> Option<Self> {
		Some(match &read_filter.inner {
			read_filter::ReadFilterInner::NewerThanLastRead(x) => {
				ReadFilter::NewerThanRead(Newer::unparse(x)?)
			}
			read_filter::ReadFilterInner::NotPresentInReadList(x) => {
				ReadFilter::NotPresentInReadList(NotPresent::unparse(x)?)
			}
		})
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Newer {
	last_read_id: String,
}

impl Newer {
	pub(crate) fn parse(self) -> read_filter::newer::Newer {
		read_filter::newer::Newer {
			last_read_id: Some(self.last_read_id),
		}
	}

	pub(crate) fn unparse(read_filter: &read_filter::newer::Newer) -> Option<Self> {
		read_filter.last_read_id.as_ref().map(|last_read_id| Self {
			last_read_id: last_read_id.clone(),
		})
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct NotPresent {
	read_list: Vec<String>,
}

impl NotPresent {
	pub(crate) fn parse(self) -> read_filter::not_present::NotPresent {
		read_filter::not_present::NotPresent {
			read_list: self.read_list.into(),
		}
	}

	pub(crate) fn unparse(read_filter: &read_filter::not_present::NotPresent) -> Option<Self> {
		if read_filter.read_list.is_empty() {
			None
		} else {
			Some(Self {
				read_list: read_filter.read_list.iter().cloned().collect(),
			})
		}
	}
}
