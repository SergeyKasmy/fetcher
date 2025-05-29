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

mod external_implementations;

pub use self::{
	external_save_wrapper::ExternalSaveRFWrapper, mark_as_read::MarkAsRead, newer::Newer,
	not_present::NotPresent,
};

use crate::{actions::filters::Filter, maybe_send::MaybeSendSync};

use std::convert::Infallible;

/// The trait that marks a type as a "read filter",
/// that allows filtering out read items out of the list of [`entries`][Entry]
/// as well as marking an [Entry] as read
///
/// [Entry]: crate::entry::Entry
pub trait ReadFilter: MarkAsRead + Filter + MaybeSendSync {}

impl<RF: ReadFilter> ReadFilter for Option<RF> {}
impl ReadFilter for () {}
impl ReadFilter for Infallible {}

#[cfg(feature = "nightly")]
impl ReadFilter for ! {}

impl<R> ReadFilter for &mut R where R: ReadFilter {}
