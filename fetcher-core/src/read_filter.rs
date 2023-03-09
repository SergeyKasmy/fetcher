/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ReadFilter`] that is used for keeping track of what Entry has been or not been read,
//! including all of its stragedies

pub mod external_implementations;
pub mod external_save;
mod newer;
mod not_present;

pub use newer::Newer;
pub use not_present::NotPresent;

use crate::{action::filter::Filter, entry::EntryId, error::Error};

use async_trait::async_trait;
use std::{any::Any, fmt::Debug};

/// A trait that defines a way to mark an entry as read
#[async_trait]
pub trait MarkAsRead: Debug + Send + Sync {
	// TODO: remake into type Err and restrict trait ReadFilter to MarkAsRead::Err: ReadFilterErr and trait Source to MarkAsRead::Err: SourceError
	/// Mark the entry with `id` as read
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Error>;

	/// Set the current "mark as read"er to read only mode
	async fn set_read_only(&mut self);
}

/// The trait that marks a type as a "read filter",
/// that allows filtering out read items out of the list of (entries)[`Entry`]
/// as well as marking an [`Entry`] as read
#[async_trait]
pub trait ReadFilter: MarkAsRead + Filter + Send + Sync {
	/// Return itself as a trait object that implements [`Any`]
	/// Used in downcasting, especially through an [ExternalSave](`external_save::ExternalSave`)
	async fn as_any(&self) -> Box<dyn Any>;
}
