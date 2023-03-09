/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use chrono::Utc;
use serde::{Deserialize, Serialize};

use fetcher_core::read_filter::{
	external_save::{
		ExternalSave as CExternalSave, ExternalSaveRFWrapper as CExternalSaveRFWrapper,
	},
	Newer as CNewer, NotPresent as CNotPresent, ReadFilter as CReadFilter,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReadFilter {
	NewerThanRead(Newer),
	NotPresentInReadList(NotPresent),
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Kind {
	NewerThanRead,
	NotPresentInReadList,
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
	pub fn parse<S>(self, external_save: S) -> Box<dyn CReadFilter>
	where
		S: CExternalSave + 'static,
	{
		match self {
			ReadFilter::NewerThanRead(rf) => Box::new(CExternalSaveRFWrapper {
				rf: rf.parse(),
				external_save: Some(external_save),
			}),
			ReadFilter::NotPresentInReadList(rf) => Box::new(CExternalSaveRFWrapper {
				rf: rf.parse(),
				external_save: Some(external_save),
			}),
		}
	}

	pub async fn unparse(read_filter: &dyn CReadFilter) -> Option<Self> {
		let any_rf = read_filter.as_any().await;

		if let Some(c_newer) = any_rf.downcast_ref::<CNewer>() {
			return Some(Self::NewerThanRead(Newer::unparse(c_newer)?));
		}

		if let Some(c_not_present) = any_rf.downcast_ref::<CNotPresent>() {
			return Some(Self::NotPresentInReadList(NotPresent::unparse(
				c_not_present,
			)?));
		}

		// FIXME: return error
		None
	}

	pub fn to_kind(&self) -> Kind {
		match self {
			ReadFilter::NewerThanRead(_) => Kind::NewerThanRead,
			ReadFilter::NotPresentInReadList(_) => Kind::NotPresentInReadList,
		}
	}
}

impl Kind {
	pub fn new_from_kind<S>(self, external_save: S) -> Box<dyn CReadFilter>
	where
		S: CExternalSave + 'static,
	{
		match self {
			Self::NewerThanRead => Box::new(CExternalSaveRFWrapper {
				rf: CNewer::new(),
				external_save: Some(external_save),
			}),
			Self::NotPresentInReadList => Box::new(CExternalSaveRFWrapper {
				rf: CNotPresent::new(),
				external_save: Some(external_save),
			}),
		}
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

impl PartialEq<Kind> for ReadFilter {
	fn eq(&self, other: &Kind) -> bool {
		self.to_kind() == *other
	}
}
