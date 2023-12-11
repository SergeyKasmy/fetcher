/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains a [`ExternalSaveRFWrapper`] that wraps a [`ReadFilter`] with an [`ExternalSave`] and implements [`ReadFilter`] itself

use async_trait::async_trait;
use std::{any::Any, fmt::Debug};

use crate::{
	action::filter::Filter,
	entry::{Entry, EntryId},
	error::FetcherError,
	external_save::ExternalSave,
	read_filter::{MarkAsRead, ReadFilter},
};

/// A wrapper that zips a [`ReadFilter`] and an [`ExternalSave`] together, implementing [`ExternalSave`] itself
/// and calling [`ExternalSave::save_read_filter`] every time [`MarkAsRead::mark_as_read`] is used
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
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError> {
		self.rf.mark_as_read(id).await?;

		if let Some(ext_save) = &mut self.external_save {
			ext_save
				.save_read_filter(&self.rf)
				.await
				.map_err(FetcherError::ExternalSave)?;
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

	fn is_readfilter(&self) -> bool {
		true
	}
}
