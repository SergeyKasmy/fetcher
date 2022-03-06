/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// 06.03.22 CONTINUE: finish NotPresentInReadList read filter and migrate html

mod newer;

use self::newer::ReadFilterNewer;
use crate::error::Result;
use crate::settings::last_read_id;

pub type Id<'a> = &'a str;

pub trait Identifiable {
	fn id(&self) -> Id;
}

#[derive(Debug)]
pub enum ReadFilter {
	NewerThanRead(ReadFilterNewer),
	NotPresentInReadList,
}

impl ReadFilter {
	pub(crate) fn read_from_fs(name: &str) -> Result<Self> {
		use ReadFilter::*;

		Ok(NewerThanRead(ReadFilterNewer::new(last_read_id(name)?)))
	}

	pub(crate) fn last_read(&self) -> Option<Id> {
		match self {
			ReadFilter::NewerThanRead(x) => x.last_read(),
			ReadFilter::NotPresentInReadList => todo!(),
		}
	}

	pub(crate) fn remove_read_from<T: Identifiable>(&self, list: &mut Vec<T>) {
		match self {
			ReadFilter::NewerThanRead(x) => x.remove_read_from(list),
			ReadFilter::NotPresentInReadList => todo!(),
		}
	}

	pub(crate) fn mark_as_read(&mut self, id: Id) {
		match self {
			ReadFilter::NewerThanRead(x) => x.mark_as_read(id),
			ReadFilter::NotPresentInReadList => todo!(),
		}
	}
}
