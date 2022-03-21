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
use crate::error::{Error, Result};

pub type Writer = Box<dyn Write + Send + Sync>;

pub struct ReadFilter {
	pub(crate) inner: ReadFilterInner,
	pub(crate) external_save: Writer,
}

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
	pub fn new(kind: Kind, external_save: Writer) -> Self {
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

	#[allow(clippy::missing_errors_doc)] // TODO
	pub(crate) async fn mark_as_read(&mut self, id: &str) -> Result<()> {
		use ReadFilterInner::{NewerThanLastRead, NotPresentInReadList};

		match &mut self.inner {
			NewerThanLastRead(x) => x.mark_as_read(id),
			NotPresentInReadList(x) => x.mark_as_read(id),
		}

		match config::read_filter::ReadFilter::unparse(self) {
			Some(filter_conf) => {
				let s = serde_json::to_string(&filter_conf).unwrap(); // unwrap NOTE: safe, serialization of such a simple struct should never fail

				// is this even worth it?
				{
					let mut w = std::mem::replace(&mut self.external_save, Box::new(Vec::new()));

					let mut w = tokio::task::spawn_blocking(move || {
						w.write_all(s.as_bytes())
							.map_err(Error::LocalIoWriteReadFilterData)?;
						Ok::<_, Error>(w)
					})
					.await
					.unwrap()?; // unwrap NOTE: crash the app if the thread crashed

					std::mem::swap(&mut w, &mut self.external_save);
				}
			}
			None => (),
		}

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
			.finish_non_exhaustive()
	}
}
