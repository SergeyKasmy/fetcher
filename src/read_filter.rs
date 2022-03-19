/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub(crate) mod newer;
pub(crate) mod not_present;

use std::io::{self, Write};

use self::newer::Newer;
use self::not_present::NotPresent;
use crate::config;
use crate::entry::Entry;
use crate::error::{Error, Result};

pub type Writer = Box<dyn Fn() -> Result<Box<dyn Write>> + Send + Sync>;

pub struct ReadFilter {
	pub(crate) inner: ReadFilterInner,
	pub(crate) external_save: Option<Writer>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ReadFilterInner {
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
	pub fn new(kind: Kind, external_save: Option<Writer>) -> Self {
		let inner = match kind {
			Kind::NewerThanLastRead => ReadFilterInner::NewerThanLastRead(Newer::new()),
			Kind::NotPresentInReadList => ReadFilterInner::NotPresentInReadList(NotPresent::new()),
		};

		Self {
			inner,
			external_save,
		}
	}

	pub(crate) fn last_read(&self) -> Option<&str> {
		use ReadFilterInner::{NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(x) => x.last_read(),
			NotPresentInReadList(x) => x.last_read(),
		}
	}

	pub(crate) fn remove_read_from(&self, list: &mut Vec<Entry>) {
		use ReadFilterInner::{NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(x) => x.remove_read_from(list),
			NotPresentInReadList(x) => x.remove_read_from(list),
		}
	}

	// TODO: move external_save inside the struct
	#[allow(clippy::missing_errors_doc)] // TODO
	pub(crate) fn mark_as_read(&mut self, id: &str) -> Result<()> {
		use ReadFilterInner::{NewerThanLastRead, NotPresentInReadList};

		match &mut self.inner {
			NewerThanLastRead(x) => x.mark_as_read(id),
			NotPresentInReadList(x) => x.mark_as_read(id),
		}

		// settings::read_filter::save(self)
		config::read_filter::ReadFilter::unparse(self)
			.map(|filter_conf| {
				self.external_save
					.as_ref()
					.map(|w| {
						let s = serde_json::to_string(&filter_conf).unwrap(); // unwrap NOTE: safe, serialization of such a simple struct should never fail
													  // FIXME: error type
						w()?.write_all(s.as_bytes())
							.expect("Read Filter save error"); // FIXME

						Ok::<(), Error>(())
					})
					.transpose()
			})
			.transpose()?;

		Ok(())
	}

	pub(crate) fn to_kind(&self) -> Kind {
		use ReadFilterInner::{NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(_) => Kind::NewerThanLastRead,
			NotPresentInReadList(_) => Kind::NotPresentInReadList,
		}
	}
}

impl std::fmt::Debug for ReadFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ReadFilter")
			.field("inner", &self.inner)
			.field("external_save.is_some", &self.external_save.is_some())
			.finish()
	}
}
