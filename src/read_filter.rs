/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub(crate) mod newer;
pub(crate) mod not_present;

use self::newer::Newer;
use self::not_present::NotPresent;
use crate::entry::Entry;
use crate::error::Result;
use crate::settings;

// TODO: dont store all this stuff if ReadFilterKind == Custom
#[derive(Debug)]
pub struct ReadFilter {
	pub(crate) name: String,
	pub(crate) inner: ReadFilterInner,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ReadFilterInner {
	NewerThanLastRead(Newer),
	NotPresentInReadList(NotPresent),
	Custom,
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
	NewerThanLastRead,
	NotPresentInReadList,
	Custom,
}

impl ReadFilter {
	// TODO: properly migrate types if the one on the disk is of one type and the provided one is of different type
	pub(crate) fn read_from_fs(name: &str, default_type: Kind) -> Result<Self> {
		settings::get(name).map(|x| {
			x.unwrap_or_else(|| {
				let inner = match default_type {
					Kind::NewerThanLastRead => ReadFilterInner::NewerThanLastRead(Newer::default()),
					Kind::NotPresentInReadList => {
						ReadFilterInner::NotPresentInReadList(NotPresent::default())
					}
					Kind::Custom => ReadFilterInner::Custom,
				};

				Self {
					name: name.to_owned(),
					inner,
				}
			})
		})
	}

	pub(crate) fn last_read(&self) -> Option<&str> {
		use ReadFilterInner::{Custom, NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(x) => x.last_read(),
			NotPresentInReadList(x) => x.last_read(),
			Custom => None,
		}
	}

	pub(crate) fn remove_read_from(&self, list: &mut Vec<Entry>) {
		use ReadFilterInner::{Custom, NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(x) => x.remove_read_from(list),
			NotPresentInReadList(x) => x.remove_read_from(list),
			Custom => (),
		}
	}

	#[allow(clippy::missing_errors_doc)] // TODO
	pub(crate) fn mark_as_read(&mut self, id: &str) -> Result<()> {
		use ReadFilterInner::{Custom, NewerThanLastRead, NotPresentInReadList};

		match &mut self.inner {
			NewerThanLastRead(x) => x.mark_as_read(id),
			NotPresentInReadList(x) => x.mark_as_read(id),
			Custom => (),
		}

		settings::save(self)
	}

	pub(crate) fn to_kind(&self) -> Kind {
		use ReadFilterInner::{Custom, NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(_) => Kind::NewerThanLastRead,
			NotPresentInReadList(_) => Kind::NotPresentInReadList,
			Custom => Kind::Custom,
		}
	}
}
