/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub(crate) mod newer;
pub(crate) mod not_present;

use self::newer::ReadFilterNewer;
use self::not_present::ReadFilterNotPresent;
use crate::error::Result;
use crate::settings;

pub type Id<'a> = &'a str;

pub trait Identifiable {
	fn id(&self) -> Id;
}

#[derive(Debug)]
pub struct ReadFilter {
	pub(crate) name: String,
	pub(crate) inner: ReadFilterInner,
}

#[derive(Debug)]
pub enum ReadFilterInner {
	NewerThanLastRead(ReadFilterNewer),
	NotPresentInReadList(ReadFilterNotPresent),
}

#[derive(Clone, Copy, Debug)]
pub enum ReadFilterKind {
	NewerThanLastRead,
	NotPresentInReadList,
}

impl ReadFilter {
	// TODO: properly migrate types if the one on the disk is of one type and the provided one is of different type
	pub(crate) fn read_from_fs(name: &str, default_type: ReadFilterKind) -> Result<Self> {
		settings::read_filter(name).map(|x| {
			x.unwrap_or_else(|| {
				let inner = match default_type {
					ReadFilterKind::NewerThanLastRead => {
						ReadFilterInner::NewerThanLastRead(ReadFilterNewer::default())
					}
					ReadFilterKind::NotPresentInReadList => {
						ReadFilterInner::NotPresentInReadList(ReadFilterNotPresent::default())
					}
				};

				Self {
					name: name.to_owned(),
					inner,
				}
			})
		})
	}

	pub(crate) fn last_read(&self) -> Option<Id> {
		match &self.inner {
			ReadFilterInner::NewerThanLastRead(x) => x.last_read(),
			ReadFilterInner::NotPresentInReadList(x) => x.last_read(),
		}
	}

	pub(crate) fn remove_read_from<T: Identifiable>(&self, list: &mut Vec<T>) {
		match &self.inner {
			ReadFilterInner::NewerThanLastRead(x) => x.remove_read_from(list),
			ReadFilterInner::NotPresentInReadList(x) => x.remove_read_from(list),
		}
	}

	pub(crate) fn mark_as_read(&mut self, id: Id) -> Result<()> {
		match &mut self.inner {
			ReadFilterInner::NewerThanLastRead(x) => x.mark_as_read(id),
			ReadFilterInner::NotPresentInReadList(x) => x.mark_as_read(id),
		}

		settings::save_read_filter(self)
	}
}
