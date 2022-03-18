/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub(crate) mod newer;
pub(crate) mod not_present;

use std::io::Write;

use self::newer::Newer;
use self::not_present::NotPresent;
use crate::config;
use crate::entry::Entry;
use crate::error::Result;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ReadFilter {
	NewerThanLastRead(Newer),
	NotPresentInReadList(NotPresent),
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
	NewerThanLastRead,
	NotPresentInReadList,
}

impl ReadFilter {
	#[must_use]
	pub fn new(kind: Kind, name: String) -> Self {
		match kind {
			Kind::NewerThanLastRead => Self::NewerThanLastRead(Newer::new(name)),
			Kind::NotPresentInReadList => Self::NotPresentInReadList(NotPresent::new(name)),
		}
	}
	// // TODO: properly migrate types if the one on the disk is of one type and the provided one is of different type
	// pub(crate) fn read_from_fs(name: String, default_type: Kind) -> Result<Self> {
	// 	// TODO
	// 	settings::read_filter::get(name.clone()).map(|x| {
	// 		x.unwrap_or_else(|| match default_type {
	// 			Kind::NewerThanLastRead => ReadFilter::NewerThanLastRead(Newer::new(name)),
	// 			Kind::NotPresentInReadList => {
	// 				ReadFilter::NotPresentInReadList(NotPresent::new(name))
	// 			}
	// 		})
	// 	})
	// }

	// pub(crate) fn delete_from_fs(self) -> Result<()> {
	// 	settings::read_filter::delete(&self)
	// }

	pub(crate) fn name(&self) -> &str {
		use ReadFilter::{NewerThanLastRead, NotPresentInReadList};

		match self {
			NewerThanLastRead(x) => &x.name,
			NotPresentInReadList(x) => &x.name,
		}
	}

	pub(crate) fn last_read(&self) -> Option<&str> {
		use ReadFilter::{NewerThanLastRead, NotPresentInReadList};

		match &self {
			NewerThanLastRead(x) => x.last_read(),
			NotPresentInReadList(x) => x.last_read(),
		}
	}

	pub(crate) fn remove_read_from(&self, list: &mut Vec<Entry>) {
		use ReadFilter::{NewerThanLastRead, NotPresentInReadList};

		match &self {
			NewerThanLastRead(x) => x.remove_read_from(list),
			NotPresentInReadList(x) => x.remove_read_from(list),
		}
	}

	// TODO: move external_save inside the struct
	#[allow(clippy::missing_errors_doc)] // TODO
	pub(crate) fn mark_as_read(&mut self, id: &str, mut external_save: impl Write) -> Result<()> {
		use ReadFilter::{NewerThanLastRead, NotPresentInReadList};

		match self {
			NewerThanLastRead(x) => x.mark_as_read(id),
			NotPresentInReadList(x) => x.mark_as_read(id),
		}

		// settings::read_filter::save(self)
		config::read_filter::ReadFilter::unparse(&self).map(|filter_conf| {
			let s = serde_json::to_string(&filter_conf).unwrap(); // unwrap NOTE: safe, serialization of such a simple struct should never fail
			external_save
				.write_all(s.as_bytes())
				.expect("Read Filter save error"); // unwrap FIXME
		});

		Ok(())
	}

	pub(crate) fn to_kind(&self) -> Kind {
		use ReadFilter::{NewerThanLastRead, NotPresentInReadList};

		match &self {
			NewerThanLastRead(_) => Kind::NewerThanLastRead,
			NotPresentInReadList(_) => Kind::NotPresentInReadList,
		}
	}
}
