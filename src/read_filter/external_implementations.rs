/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Implementations of [`ReadFilter`] on foreign types.
//! These should make passing your own [`ReadFilter`] types easier without having to make an newtype just to implement it yourself

use std::sync::Arc;
use tokio::sync::Mutex;

use super::{MarkAsRead, ReadFilter};
use crate::{
	actions::filters::Filter,
	entry::{Entry, EntryId},
	error::FetcherError,
};

/// [`ReadFilter`] implementation for `Arc<tokio::Mutex<impl ReadFilter>>`
pub mod tokio_mutex {
	#[allow(clippy::wildcard_imports)]
	use super::*;

	impl<RF> ReadFilter for Arc<Mutex<RF>> where RF: ReadFilter {}

	impl<RF> MarkAsRead for Arc<Mutex<RF>>
	where
		RF: ReadFilter,
	{
		async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError> {
			self.lock().await.mark_as_read(id).await
		}

		async fn set_read_only(&mut self) {
			self.lock().await.set_read_only().await;
		}
	}

	impl<RF> Filter for Arc<Mutex<RF>>
	where
		RF: ReadFilter,
	{
		type Err = RF::Err;

		async fn filter(&mut self, entries: &mut Vec<Entry>) -> Result<(), Self::Err> {
			self.lock().await.filter(entries).await
		}
	}
}

/*
// TODO: is this needed?
/// [`ReadFilter`] implementation for `Box<dyn ReadFilter>`
pub mod boks {
	#[allow(clippy::wildcard_imports)]
	use super::*;

	#[async_trait]
	impl ReadFilter for Box<dyn ReadFilter> {}

	#[async_trait]
	impl MarkAsRead for Box<dyn ReadFilter> {
		async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError> {
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

		fn is_readfilter(&self) -> bool {
			true
		}
	}
}
*/
