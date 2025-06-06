/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ReadFilter`] that is used for keeping track of what Entry has been or not been read,
//! including all of its stragedies

pub mod mark_as_read;

mod external_save_wrapper;
mod newer;
mod not_present;
mod shared;

pub use self::{
	external_save_wrapper::ExternalSaveRFWrapper, mark_as_read::MarkAsRead, newer::Newer,
	not_present::NotPresent, shared::Shared,
};

use crate::{actions::filters::Filter, external_save::ExternalSave, maybe_send::MaybeSendSync};

use std::convert::Infallible;

/// The trait that marks a type as a "read filter",
/// that allows filtering out read items out of the list of [`entries`][Entry]
/// as well as marking an [Entry] as read
///
/// [Entry]: crate::entry::Entry
pub trait ReadFilter: MarkAsRead + Filter + MaybeSendSync {
	/// Wraps current read filter in [`Shared`]
	fn into_shared(self) -> Shared<Self>
	where
		Self: Sized,
	{
		Shared::new(self)
	}

	/// Attaches the provided external save implementation to the current read filter
	/// to be called on each on each [`MarkAsRead::mark_as_read`].
	fn externally_saved<S: ExternalSave>(self, save: S) -> ExternalSaveRFWrapper<Self, S>
	where
		Self: Sized,
	{
		ExternalSaveRFWrapper::new(self, save)
	}
}

impl<RF: ReadFilter> ReadFilter for Option<RF> {}
impl ReadFilter for () {}
impl ReadFilter for Infallible {}

#[cfg(feature = "nightly")]
impl ReadFilter for ! {}

impl<R> ReadFilter for &mut R where R: ReadFilter {}
