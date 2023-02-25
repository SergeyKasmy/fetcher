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
use std::{any::Any, sync::Arc};
use tokio::sync::RwLock;

use crate::{action::filter::Filter, entry::Entry, error::Error, source::MarkAsRead};

/// The trait that marks a type as a "read filter",
/// that allows filtering out read items out of the list of (entries)[`Entry`]
/// as well as marking an [`Entry`] as read
#[async_trait]
pub trait ReadFilter: MarkAsRead + Filter + Send + Sync {
	/// Return itself as a trait object that implements [`Any`]
	/// Used in downcasting, especially through an [ExternalSave](`external_save::ExternalSave`)
	async fn as_any(&self) -> Box<dyn Any>;
}

#[async_trait]
impl<RF> ReadFilter for Arc<RwLock<RF>>
where
	RF: ReadFilter,
{
	async fn as_any(&self) -> Box<dyn Any> {
		self.read().await.as_any().await
	}
}

#[async_trait]
impl<RF> MarkAsRead for Arc<RwLock<RF>>
where
	RF: ReadFilter,
{
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		self.write().await.mark_as_read(id).await
	}

	async fn set_read_only(&mut self) {
		self.write().await.set_read_only().await;
	}
}

#[async_trait]
impl<RF> Filter for Arc<RwLock<RF>>
where
	RF: ReadFilter,
{
	async fn filter(&self, entries: &mut Vec<Entry>) {
		self.read().await.filter(entries).await;
	}
}

#[async_trait]
impl ReadFilter for Box<dyn ReadFilter> {
	async fn as_any(&self) -> Box<dyn Any> {
		(**self).as_any().await
	}
}

#[async_trait]
impl MarkAsRead for Box<dyn ReadFilter> {
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		(**self).mark_as_read(id).await
	}

	async fn set_read_only(&mut self) {
		(**self).set_read_only().await;
	}
}

#[async_trait]
impl Filter for Box<dyn ReadFilter> {
	async fn filter(&self, entries: &mut Vec<Entry>) {
		(**self).filter(entries).await;
	}
}
