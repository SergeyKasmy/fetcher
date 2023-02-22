/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use async_trait::async_trait;
use std::{any::Any, fmt::Debug};

use super::ReadFilter;
use crate::{action::filter::Filter, entry::Entry, error::Error, source::MarkAsRead};

/// This trait represent some kind of external save destination.
/// A way to preserve the state of a read filter, i.e. what has and has not been read, across restarts.
#[async_trait]
pub trait ExternalSave: Debug + Send + Sync {
	/// This function will be called every time something has been marked as read and should be saved externally
	///
	/// # Errors
	/// It may return an error if there has been issues saving, e.g. writing to disk
	// TODO: trait for deserializing instead of impl ReadFilter
	async fn save(&mut self, read_filter: &dyn ReadFilter) -> std::io::Result<()>;
}

#[derive(Debug)]
pub struct ExternalSaveRFWrapper {
	pub rf: Box<dyn ReadFilter>,
	pub external_save: Option<Box<dyn ExternalSave>>,
}

#[async_trait]
impl ReadFilter for ExternalSaveRFWrapper {
	async fn as_any(&self) -> Box<dyn Any> {
		self.rf.as_any().await
	}
}

#[async_trait]
impl MarkAsRead for ExternalSaveRFWrapper {
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		self.rf.mark_as_read(id).await?;

		if let Some(ext_save) = &mut self.external_save {
			ext_save
				.save(&*self.rf)
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
impl Filter for ExternalSaveRFWrapper {
	async fn filter(&self, entries: &mut Vec<Entry>) {
		self.rf.filter(entries).await;
	}
}
