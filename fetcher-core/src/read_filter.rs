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

use crate::{action::filter::Filter, source::MarkAsRead};

use async_trait::async_trait;
use std::any::Any;

/// The trait that marks a type as a "read filter",
/// that allows filtering out read items out of the list of (entries)[`Entry`]
/// as well as marking an [`Entry`] as read
#[async_trait]
pub trait ReadFilter: MarkAsRead + Filter + Send + Sync {
	/// Return itself as a trait object that implements [`Any`]
	/// Used in downcasting, especially through an [ExternalSave](`external_save::ExternalSave`)
	async fn as_any(&self) -> Box<dyn Any>;
}
