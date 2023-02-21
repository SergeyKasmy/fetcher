/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`TransformEntry`](`entry::TransformEntry`) and [`TransformField`] traits as well as all types that implement it

pub mod entry;
pub mod field;
pub mod result;

pub use self::{
	entry::{feed::Feed, html::Html, json::Json, use_as::Use},
	field::{caps::Caps, shorten::Shorten, trim::Trim},
};

use crate::{entry::Entry, error::transform::Error as TransformError};

use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait Transform: Debug {
	async fn transform(&self, entry: &Entry) -> Result<Vec<Entry>, TransformError>;
}
