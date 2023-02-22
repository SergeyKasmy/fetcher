/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use chrono::Utc;
use serde::{Deserialize, Serialize};

use fetcher_core::read_filter::{
	ExternalSave as CExternalSave, Newer as CNewer, NotPresent as CNotPresent,
	ReadFilter as CReadFilter,
};

#[derive(Deserialize, Serialize, Clone, Copy, Debug)]
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

impl ReadFilter {
	pub fn parse(
		self,
		external_save: Box<dyn CExternalSave + Send + Sync>,
	) -> Box<dyn CReadFilter> {
		let inner: Box<dyn CReadFilter> = match self {
			ReadFilter::NewerThanRead(x) => Box::new(x.parse()),
			ReadFilter::NotPresentInReadList(x) => Box::new(x.parse()),
		};

		/*
		core_rf::ReadFilter {
			inner,
			external_save: Some(external_save),
		}
		*/
		inner
	}

	pub fn unparse(read_filter: &impl CReadFilter) -> Option<Self> {
		todo!()
		// Some(match &read_filter {
		// 	core_rf::Inner::NewerThanLastRead(x) => ReadFilter::NewerThanRead(Newer::unparse(x)?),
		// 	core_rf::Inner::NotPresentInReadList(x) => {
		// 		ReadFilter::NotPresentInReadList(NotPresent::unparse(x)?)
		// 	}
		// })
	}
}

impl Newer {
	pub fn parse(self) -> CNewer {
		CNewer {
			last_read_id: Some(self.last_read_id),
		}
	}

	pub fn unparse(read_filter: &CNewer) -> Option<Self> {
		read_filter.last_read_id.as_ref().map(|last_read_id| Self {
			last_read_id: last_read_id.clone(),
		})
	}
}

impl NotPresent {
	pub fn parse(self) -> CNotPresent {
		CNotPresent::from_iter(self.read_list)
	}

	pub fn unparse(read_filter: &CNotPresent) -> Option<Self> {
		if read_filter.is_empty() {
			None
		} else {
			Some(Self {
				read_list: read_filter.iter().cloned().collect(),
			})
		}
	}
}
