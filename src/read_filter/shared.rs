/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::sync::Arc;

use tokio::sync::Mutex as TokioMutex;

use crate::{
	actions::filters::Filter,
	entry::{Entry, EntryId},
};

use super::{MarkAsRead, ReadFilter};

/// Wrapper around a [`ReadFilter`] with support for sharing via [`Arc`].
///
/// Read filters are intended to be used both as a part of a source (to mark entries as read), and as a filter (to filter read entries).
/// This is the indended way to provide access to a single read filter instance to both.
#[derive(Debug)]
pub struct Shared<RF>(Arc<TokioMutex<RF>>);

impl<RF> Shared<RF> {
	/// Creates an instance of [`Shared`] containing the provided read filter
	pub fn new(rf: RF) -> Self {
		Self(Arc::new(TokioMutex::new(rf)))
	}
}

impl<RF: MarkAsRead> MarkAsRead for Shared<RF> {
	type Err = RF::Err;

	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Self::Err> {
		self.0.lock().await.mark_as_read(id).await
	}

	async fn set_read_only(&mut self) {
		self.0.lock().await.set_read_only().await
	}
}

impl<RF: Filter> Filter for Shared<RF> {
	type Err = RF::Err;

	async fn filter(&mut self, entries: &mut Vec<Entry>) -> Result<(), Self::Err> {
		self.0.lock().await.filter(entries).await
	}
}
impl<RF: ReadFilter> ReadFilter for Shared<RF> {}

impl<RF> Clone for Shared<RF> {
	fn clone(&self) -> Self {
		Self(Arc::clone(&self.0))
	}
}
