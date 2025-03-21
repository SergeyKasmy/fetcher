/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ReadFilter`] that is used for keeping track of what Entry has been or not been read,
//! including all of its stragedies

mod external_save_wrapper;
mod newer;
mod not_present;

mod external_implementations;

pub use self::{
	external_save_wrapper::ExternalSaveRFWrapper, newer::Newer, not_present::NotPresent,
};

use crate::{action::filters::Filter, entry::EntryId, error::FetcherError};

use std::fmt::Debug;

/// A trait that defines a way to mark an entry as read
pub trait MarkAsRead: Debug + Send + Sync {
	// TODO: remake into type Err and restrict trait ReadFilter to MarkAsRead::Err: ReadFilterErr and trait Source to MarkAsRead::Err: SourceError
	/// Mark the entry with `id` as read
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError>;

	/// Set the current "mark as read"er to read only mode
	async fn set_read_only(&mut self);
}

/// The trait that marks a type as a "read filter",
/// that allows filtering out read items out of the list of [`entries`][Entry]
/// as well as marking an [Entry] as read
///
/// [Entry]: crate::entry::Entry
pub trait ReadFilter: MarkAsRead + Filter + Send + Sync {}

impl<M: MarkAsRead> MarkAsRead for Option<M> {
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError> {
		match self {
			Some(m) => m.mark_as_read(id).await?,
			None => {
				tracing::debug!("Ignoring mark as read request");
			}
		}

		Ok(())
	}

	async fn set_read_only(&mut self) {
		match self {
			Some(m) => m.set_read_only().await,
			None => {
				tracing::debug!("Ignoring set read only request");
			}
		}
	}
}

impl<RF: ReadFilter> ReadFilter for Option<RF> {}

/*
impl MarkAsRead for () {
	async fn mark_as_read(&mut self, _id: &EntryId) -> Result<(), FetcherError> {
		tracing::debug!("Ignoring mark as read request on purpose");
		Ok(())
	}

	/// Set the current "mark as read"er to read only mode
	async fn set_read_only(&mut self) {}
}

impl ReadFilter for () {}
*/
