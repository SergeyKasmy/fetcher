/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ExternalSave`] trait that implementors can use to add a way to save read filter data externally,
//! as well as [`ExternalSaveRFWrapper`] that wraps a [`ReadFilter`] with an [`ExternalSave`] and implements [`ReadFilter`] itself

use async_trait::async_trait;
use std::{any::Any, fmt::Debug};

use super::ReadFilter;
use crate::{
	action::filter::Filter,
	entry::{Entry, EntryId},
	error::Error,
	source::MarkAsRead,
};

/// This trait represent some kind of external save destination.
/// A way to preserve the state of a read filter, i.e. what has and has not been read, across restarts.
#[async_trait]
pub trait ExternalSave: Debug + Send + Sync {
	/// This function will be called every time something has been marked as read and should be saved externally
	///
	/// # Errors
	/// It may return an error if there has been issues saving, e.g. writing to disk
	// TODO: trait for deserializing instead of dyn ReadFilter
	async fn save(&mut self, read_filter: &dyn ReadFilter) -> std::io::Result<()>;
}

/// A wrapper that zips a [`ReadFilter`] and an [`ExternalSave`] together, implementing [`ExternalSave`] itself
/// and calling [`ExternalSave::save`] every time [`MarkAsRead::mark_as_read`] is used
#[derive(Debug)]
pub struct ExternalSaveRFWrapper<RF, S> {
	/// The [`ReadFilter`] that is being wrapped
	pub rf: RF,
	/// The [`ExternalSave`] that is being called on each call to [`MarkAsRead::mark_as_read`]
	pub external_save: Option<S>,
}

#[async_trait]
impl<RF, S> ReadFilter for ExternalSaveRFWrapper<RF, S>
where
	RF: ReadFilter,
	S: ExternalSave,
{
	async fn as_any(&self) -> Box<dyn Any> {
		self.rf.as_any().await
	}
}

#[async_trait]
impl<RF, S> MarkAsRead for ExternalSaveRFWrapper<RF, S>
where
	RF: ReadFilter,
	S: ExternalSave,
{
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Error> {
		self.rf.mark_as_read(id).await?;

		if let Some(ext_save) = &mut self.external_save {
			ext_save
				.save(&self.rf)
				.await
				.map_err(Error::ReadFilterExternalWrite)?;
		}

		Ok(())
	}

	async fn set_read_only(&mut self) {
		self.external_save = None;
	}
}

#[async_trait]
impl<RF, S> Filter for ExternalSaveRFWrapper<RF, S>
where
	RF: ReadFilter,
	S: ExternalSave,
{
	async fn filter(&self, entries: &mut Vec<Entry>) {
		self.rf.filter(entries).await;
	}
}
