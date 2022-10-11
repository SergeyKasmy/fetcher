/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use chrono::Utc;
use serde::{Deserialize, Serialize};

use fetcher_core::read_filter as core_rf;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Kind {
	NewerThanRead,
	NotPresentInReadList,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReadFilter {
	NewerThanRead(Newer),
	NotPresentInReadList(NotPresent),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Newer {
	last_read_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NotPresent {
	read_list: Vec<(String, chrono::DateTime<Utc>)>,
}

impl Kind {
	pub fn parse(self) -> core_rf::Kind {
		match self {
			Kind::NewerThanRead => core_rf::Kind::NewerThanLastRead,
			Kind::NotPresentInReadList => core_rf::Kind::NotPresentInReadList,
		}
	}
}

impl ReadFilter {
	pub fn parse(
		self,
		external_save: Box<dyn core_rf::ExternalSave + Send + Sync>,
	) -> core_rf::ReadFilter {
		let inner = match self {
			ReadFilter::NewerThanRead(x) => core_rf::Inner::NewerThanLastRead(x.parse()),
			ReadFilter::NotPresentInReadList(x) => core_rf::Inner::NotPresentInReadList(x.parse()),
		};

		core_rf::ReadFilter {
			inner,
			external_save,
		}
	}

	pub fn unparse(read_filter: &core_rf::Inner) -> Option<Self> {
		Some(match &read_filter {
			core_rf::Inner::NewerThanLastRead(x) => ReadFilter::NewerThanRead(Newer::unparse(x)?),
			core_rf::Inner::NotPresentInReadList(x) => {
				ReadFilter::NotPresentInReadList(NotPresent::unparse(x)?)
			}
		})
	}
}

impl Newer {
	pub fn parse(self) -> core_rf::Newer {
		core_rf::Newer {
			last_read_id: Some(self.last_read_id),
		}
	}

	pub fn unparse(read_filter: &core_rf::Newer) -> Option<Self> {
		read_filter.last_read_id.as_ref().map(|last_read_id| Self {
			last_read_id: last_read_id.clone(),
		})
	}
}

impl NotPresent {
	pub fn parse(self) -> core_rf::NotPresent {
		core_rf::NotPresent::from_iter(self.read_list)
	}

	pub fn unparse(read_filter: &core_rf::NotPresent) -> Option<Self> {
		if read_filter.is_empty() {
			None
		} else {
			Some(Self {
				read_list: read_filter.iter().cloned().collect(),
			})
		}
	}
}
