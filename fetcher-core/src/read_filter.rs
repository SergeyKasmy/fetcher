/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod mark_as_read;
pub mod newer;
pub mod not_present;

use self::newer::Newer;
use self::not_present::NotPresent;
use crate::entry::Entry;
use crate::error::Error;

/// This trait represent some kind of external save destination.
/// A way to preserve the state of a read filter, i.e. what has and has not been read, across restarts.
pub trait ExternalSave {
	/// This function will be called every time something has been marked as read and should be saved externally
	///
	/// # Errors
	/// It may return an error if there has been issues saving, e.g. writing to disk
	fn save(&mut self, read_filter: &ReadFilterInner) -> std::io::Result<()>;
}

// TODO: add field `since` that marks the first time that read filter was used and ignores everything before
pub struct ReadFilter {
	pub inner: ReadFilterInner,
	pub external_save: Box<dyn ExternalSave + Send + Sync>,
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
	pub fn new(kind: Kind, external_save: Box<dyn ExternalSave + Send + Sync>) -> Self {
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

	#[allow(dead_code)] // TODO
	pub(crate) fn to_kind(&self) -> Kind {
		use ReadFilterInner::{NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(_) => Kind::NewerThanLastRead,
			NotPresentInReadList(_) => Kind::NotPresentInReadList,
		}
	}

	#[allow(clippy::missing_errors_doc)] // TODO
	pub(crate) async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		use ReadFilterInner::{NewerThanLastRead, NotPresentInReadList};

		tracing::trace!("Marking {id} as read");

		match &mut self.inner {
			NewerThanLastRead(x) => x.mark_as_read(id),
			NotPresentInReadList(x) => x.mark_as_read(id),
		}

		self.external_save
			.save(&self.inner)
			.map_err(Error::ReadFilterExternalWrite)
	}

	pub(crate) fn is_unread(&self, id: &str) -> bool {
		match &self.inner {
			ReadFilterInner::NewerThanLastRead(_) => todo!(),
			ReadFilterInner::NotPresentInReadList(x) => x.is_unread(id),
		}
	}

	pub(crate) fn transform(&self, entry: &Entry) -> Vec<Entry> {
		tracing::trace!("Transforming/filtering entry id: {:?}", entry.id);

		match entry.id.as_deref() {
			Some(id) if self.is_unread(id) => vec![entry.clone()],
			None => vec![entry.clone()],
			_ => Vec::new(),
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
