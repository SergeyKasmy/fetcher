/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`TransformEntry`](`entry::TransformEntry`) and [`TransformField`](`field::TransformField`) traits as well as all types that implement it

pub mod entry;
pub mod field;
pub mod result;

pub mod error;

pub use self::{
	entry::{feed::Feed, html::Html, http::Http, json::Json, print::DebugPrint, use_as::Use},
	field::{caps::Caps, set::Set, shorten::Shorten, trim::Trim},
};

use self::error::TransformError;
use crate::{entry::Entry, external_save::ExternalSave, source::Source};

use async_trait::async_trait;
use std::fmt::Debug;

use super::{Action, ActionContext};

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
	async fn transform(&self, entry: Entry) -> Result<Vec<Entry>, TransformError>;
}

pub(crate) struct TransformWrapper<T>(pub T);

impl<T> Action for TransformWrapper<T>
where
	T: Transform,
{
	type Error = TransformError;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		_ctx: ActionContext<'_, S, E>,
	) -> Result<Vec<Entry>, Self::Error>
	where
		S: Source,
		E: ExternalSave,
	{
		let mut transformed_entries = Vec::new();

		for entry in entries {
			transformed_entries.extend(self.0.transform(entry).await?);
		}

		Ok(transformed_entries)
	}
}
