/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Implementations of [`ReadFilter`] on foreign types.
//! These should make passing your own [`ReadFilter`] types easier without having to make an newtype just to implement it yourself

use async_trait::async_trait;
use std::{any::Any, sync::Arc};
use tokio::sync::RwLock;

use super::ReadFilter;
use crate::{
	action::filter::Filter,
	entry::{Entry, EntryId},
	error::Error,
	source::MarkAsRead,
};

/// [`ReadFilter`] implementation for `Arc<tokio::RwLock<dyn Readfilter>>`
pub mod tokio_rwlock {

	#[allow(clippy::wildcard_imports)]
	use super::*;

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
		async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Error> {
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
}

/// [`ReadFilter`] implementation for `Box<dyn ReadFilter>`
pub mod boks {
	#[allow(clippy::wildcard_imports)]
	use super::*;

	#[async_trait]
	impl ReadFilter for Box<dyn ReadFilter> {
		async fn as_any(&self) -> Box<dyn Any> {
			(**self).as_any().await
		}
	}

	#[async_trait]
	impl MarkAsRead for Box<dyn ReadFilter> {
		async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Error> {
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
}
