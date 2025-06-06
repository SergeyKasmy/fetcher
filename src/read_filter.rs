/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// FIXME: docs
//! This module contains the [`ReadFilter`] that is used for keeping track of what Entry has been or not been read,
//! including all of its stragedies

pub mod mark_as_read;

mod newer;
mod not_present;

pub use self::{mark_as_read::MarkAsRead, newer::Newer, not_present::NotPresent};

use self::mark_as_read::MarkAsReadError;
use crate::{
	actions::filters::Filter,
	entry::{Entry, EntryId},
	external_save::ExternalSave,
	maybe_send::MaybeSend,
};

use std::{convert::Infallible, fmt::Debug};

use serde::Serialize;
use tokio::sync::Mutex as TokioMutex;

#[cfg(feature = "send")]
type RefCounted<T> = std::sync::Arc<T>;
#[cfg(not(feature = "send"))]
type RefCounted<T> = std::rc::Rc<T>;

// TODO: add example
/// A reference-counted read-filter wrapper.
/// Supports externally saving the inner read-filter's state if an [`ExternalSave`] is provided.
///
/// # Note
/// This is the expected way to use a read-filter in fetcher jobs.
/// [`ReadFilter`] makes it easy to combine a read-filter implementation with a read-filter-less fetch
/// and to actually filter out read entries with the read-filter.
///
/// `WITH_EXTERNAL_SAVE` specifies if [`ReadFilter`] should automatically save internal read-filter's state or not.\
/// If it's `true`, then a [`Serialize`] bound is added to the inner read-filter
/// and [`ExternalSave::save_read_filter`] is called with the inner read-filter every time [`MarkAsRead::mark_as_read`] is called.
#[derive(Debug)]
pub struct ReadFilter<T, const WITH_EXTERNAL_SAVE: bool, S = Infallible>(
	RefCounted<TokioMutex<ReadFilterInner<T, WITH_EXTERNAL_SAVE, S>>>,
);

#[derive(Debug)]
struct ReadFilterInner<T, const WITH_EXTERNAL_SAVE: bool, S = Infallible> {
	read_filter: T,
	/// Set to `None` when the read filter is set to read_only
	external_save: Option<S>,
}

impl<T, S> ReadFilter<T, true, S>
where
	T: MarkAsRead + Filter,
	S: ExternalSave,
{
	/// Creates a new [`ReadFilter`] with support for external saving via the provided `external_save`
	pub fn new(read_filter: T, external_save: S) -> Self {
		Self(RefCounted::new(TokioMutex::new(ReadFilterInner {
			read_filter,
			external_save: Some(external_save),
		})))
	}
}

impl<T: MarkAsRead + Filter> ReadFilter<T, false> {
	/// Creates a new [`ReadFilter`] without support for external saving.
	///
	/// This isn't very useful as all state will be lost when the program is restarted.
	pub fn without_external_save(read_filter: T) -> Self {
		Self(RefCounted::new(TokioMutex::new(ReadFilterInner {
			read_filter,
			external_save: None,
		})))
	}
}

impl<T, S> MarkAsRead for ReadFilter<T, true, S>
where
	T: MarkAsRead + Serialize,
	S: ExternalSave,
{
	type Err = MarkAsReadError;

	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Self::Err> {
		let mut inner = self.0.lock().await;

		inner
			.read_filter
			.mark_as_read(id)
			.await
			.map_err(Into::into)?;

		let ReadFilterInner {
			read_filter,
			external_save,
		} = &mut *inner;

		if let Some(ext_save) = external_save {
			ext_save.save_read_filter(read_filter).await?;
		}

		Ok(())
	}

	async fn set_read_only(&mut self) {
		let mut inner = self.0.lock().await;
		inner.read_filter.set_read_only().await;
		inner.external_save = None;
	}
}

impl<T> MarkAsRead for ReadFilter<T, false>
where
	T: MarkAsRead,
{
	type Err = T::Err;

	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Self::Err> {
		self.0.lock().await.read_filter.mark_as_read(id).await
	}

	async fn set_read_only(&mut self) {
		self.0.lock().await.read_filter.set_read_only().await
	}
}

impl<T, const WITH_EXTERNAL_SAVE: bool, S> Clone for ReadFilter<T, WITH_EXTERNAL_SAVE, S> {
	fn clone(&self) -> Self {
		Self(RefCounted::clone(&self.0))
	}
}

impl<T, const WITH_EXTERNAL_SAVE: bool, S> Filter for ReadFilter<T, WITH_EXTERNAL_SAVE, S>
where
	T: Filter,
	S: MaybeSend,
{
	type Err = T::Err;

	async fn filter(&mut self, entries: &mut Vec<Entry>) -> Result<(), Self::Err> {
		self.0.lock().await.read_filter.filter(entries).await
	}
}
