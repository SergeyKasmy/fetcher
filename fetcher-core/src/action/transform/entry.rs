/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`TransformEntry`] trait as well as every type that implement it

pub mod feed;
pub mod html;
pub mod json;
pub mod print;
pub mod use_as;

use async_trait::async_trait;

use super::{result::TransformedEntry, Transform};
use crate::{
	entry::Entry,
	error::transform::{Error as TransformError, Kind as TransformErrorKind},
};

use std::fmt::Debug;

pub trait TransformEntry: Debug {
	type Err: Into<TransformErrorKind>;

	/// Transform the `entry` into one or several separate entries
	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Err>;
}

#[async_trait]
impl<T> Transform for T
where
	T: TransformEntry + Sync,
{
	async fn transform(&self, entry: &Entry) -> Result<Vec<Entry>, TransformError> {
		self.transform_entry(entry)
			.map(|v| v.into_iter().map(|e| e.into_entry(entry.clone())).collect())
			.map_err(|kind| TransformError {
				kind: kind.into(),
				original_entry: entry.clone(),
			})
	}
}
