/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ReadFilter`] that is used for keeping track of what Entry has been or not been read,
//! including all of its stragedies

pub mod external_save;
mod newer;
mod not_present;

pub use newer::Newer;
pub use not_present::NotPresent;

use async_trait::async_trait;
use std::{any::Any, fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

use crate::{action::filter::Filter, entry::Entry, error::Error, source::MarkAsRead};

#[async_trait]
pub trait ReadFilter: MarkAsRead + Filter + Send + Sync {
	async fn as_any(&self) -> Box<dyn Any>;
}

#[async_trait]
impl ReadFilter for Arc<RwLock<dyn ReadFilter>> {
	async fn as_any(&self) -> Box<dyn Any> {
		self.read().await.as_any().await
	}
}

#[async_trait]
impl MarkAsRead for Arc<RwLock<dyn ReadFilter>> {
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		self.write().await.mark_as_read(id).await
	}

	async fn set_read_only(&mut self) {
		self.write().await.set_read_only().await;
	}
}

#[async_trait]
impl Filter for Arc<RwLock<dyn ReadFilter>> {
	async fn filter(&self, entries: &mut Vec<Entry>) {
		self.read().await.filter(entries).await;
	}
}

/*
#[derive(Debug)]
pub struct ReadFilterWrapper(pub Arc<RwLock<dyn ReadFilter + Send + Sync>>);

impl ReadFilter for ReadFilterWrapper {}

#[async_trait]
impl MarkAsRead for ReadFilterWrapper {
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		self.0.write().await.mark_as_read(id).await
	}
}

#[async_trait]
impl Filter for ReadFilterWrapper {
	async fn filter(&self, entries: &mut Vec<Entry>) {
		self.0.read().await.filter(entries).await;
	}
}

impl Clone for ReadFilterWrapper {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}
*/

/*
/// A built-in read filter that uses any of the
// TODO: add field `since` that marks the first time that read filter was used and ignores everything before
pub struct ReadFilter {
	#[allow(missing_docs)]
	pub inner: Inner,

	/// An external save destination
	pub external_save: Option<Box<dyn ExternalSave + Send + Sync>>,
}

/// All different read filtering stragedies
// TODO: make private?
#[allow(missing_docs)]
#[derive(Debug)]
pub enum Inner {
	NewerThanLastRead(Newer),
	NotPresentInReadList(NotPresent),
}

/// A list of all supported read filtering stragedies
#[allow(missing_docs)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Kind {
	NewerThanLastRead,
	NotPresentInReadList,
}


impl ReadFilter {
	/// Creates a new Read Filter using [`kind`](`Kind`) filter stragedy and `external_save` external saving implementation
	#[must_use]
	pub fn new(kind: Kind, external_save: Option<Box<dyn ExternalSave + Send + Sync>>) -> Self {
		let inner = match kind {
			Kind::NewerThanLastRead => Inner::NewerThanLastRead(Newer::new()),
			Kind::NotPresentInReadList => Inner::NotPresentInReadList(NotPresent::new()),
		};

		Self {
			inner,
			external_save,
		}
	}

	/// Returns the current read filtering stragedy
	#[must_use]
	pub fn to_kind(&self) -> Kind {
		use Inner::{NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(_) => Kind::NewerThanLastRead,
			NotPresentInReadList(_) => Kind::NotPresentInReadList,
		}
	}

	/*
	pub(crate) fn last_read(&self) -> Option<&str> {
		use Inner::{NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(x) => x.last_read(),
			NotPresentInReadList(x) => x.last_read(),
		}
	}
	*/

	pub(crate) fn remove_read_from(&self, list: &mut Vec<Entry>) {
		use Inner::{NewerThanLastRead, NotPresentInReadList};

		match &self.inner {
			NewerThanLastRead(x) => x.remove_read_from(list),
			NotPresentInReadList(x) => x.remove_read_from(list),
		}
	}

	pub(crate) fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		use Inner::{NewerThanLastRead, NotPresentInReadList};

		tracing::trace!("Marking {id} as read");

		match &mut self.inner {
			NewerThanLastRead(x) => x.mark_as_read(id),
			NotPresentInReadList(x) => x.mark_as_read(id),
		}

		if let Some(external_save) = &mut self.external_save {
			external_save
				.save(&self.inner)
				.map_err(Error::ReadFilterExternalWrite)?;
		}

		Ok(())
	}
}

#[async_trait]
impl Filter for ReadFilter {
	async fn filter(&self, entries: &mut Vec<Entry>) {
		self.remove_read_from(entries);
	}
}

impl std::fmt::Debug for ReadFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ReadFilter")
			.field("inner", &self.inner)
			.finish_non_exhaustive()
	}
}

impl std::fmt::Display for Kind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			Kind::NewerThanLastRead => "Newer than the last one read",
			Kind::NotPresentInReadList => "Not present in the marked as read list",
		})
	}
}
*/
