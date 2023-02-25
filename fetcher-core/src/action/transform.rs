/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`TransformEntry`](`entry::TransformEntry`) and [`TransformField`](`field::TransformField`) traits as well as all types that implement it

pub mod entry;
pub mod field;
pub mod result;

pub use self::{
	entry::{feed::Feed, html::Html, http::Http, json::Json, print::DebugPrint, use_as::Use},
	field::{caps::Caps, set::Set, shorten::Shorten, trim::Trim},
};

use crate::{entry::Entry, error::transform::Error as TransformError};

use async_trait::async_trait;
use std::fmt::Debug;

/// Transform an [`Entry`] into one or more new (entries)[`Entry`].
///
/// For example, a [`Json`] transform parses the contents of the [`Entry`] as JSON and returns new entries from it,
/// while the [`Caps`] field transform just makes a field uppercase
#[async_trait]
pub trait Transform: Debug + Send + Sync {
	/// Transform an [`Entry`] to one or more entries
	///
	/// # Erorrs
	/// Refer to implementators docs
	async fn transform(&self, entry: &Entry) -> Result<Vec<Entry>, TransformError>;
}
