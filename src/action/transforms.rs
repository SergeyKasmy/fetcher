/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`TransformEntry`](`entry::TransformEntry`) and [`TransformField`](`field::TransformField`) traits as well as all types that implement it

pub mod async_fn;
pub mod feed;
pub mod html;
pub mod http;
pub mod json;
pub mod print;
pub mod use_as;

pub mod field;
pub mod result;

pub mod error;

pub use self::{
	feed::Feed,
	field::{caps::Caps, set::Set, shorten::Shorten, trim::Trim},
	html::Html,
	http::Http,
	json::Json,
	print::DebugPrint,
	use_as::Use,
};

use self::error::TransformError;
use self::error::TransformErrorKind;
use self::result::TransformedEntry;
use crate::{
	entry::Entry,
	external_save::ExternalSave,
	maybe_send::{MaybeSend, MaybeSendSync},
	sources::Source,
};

use super::{Action, ActionContext};

/*
/// Transform an [`Entry`] into one or more new (entries)[`Entry`].
///
/// For example, a [`Json`] transform parses the contents of the [`Entry`] as JSON and returns new entries from it,
/// while the [`Caps`] field transform just makes a field uppercase
pub trait Transform: Debug + Send + Sync {
	/// Transform an [`Entry`] to one or more entries
	///
	/// # Erorrs
	/// Refer to implementators docs
	async fn transform(&self, entry: Entry) -> Result<Vec<Entry>, TransformError>;
}
*/

/// Transform an entry into one or more entries. This is the type transforms should implement as it includes easier error management
pub trait Transform: MaybeSendSync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<TransformErrorKind>;

	/// Transform the `entry` into one or several separate entries
	fn transform_entry(
		&self,
		entry: Entry,
	) -> impl Future<Output = Result<Vec<TransformedEntry>, Self::Err>> + MaybeSend;
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
			transformed_entries.extend(transform_old_entry_into_new_entries(&self.0, entry).await?);
		}

		Ok(transformed_entries)
	}
}

async fn transform_old_entry_into_new_entries<T>(
	this: &T,
	old_entry: Entry,
) -> Result<Vec<Entry>, TransformError>
where
	T: Transform,
{
	this.transform_entry(old_entry.clone())
		.await
		.map(|vec| {
			vec.into_iter()
				.map(|transformed_entry| transformed_entry.into_entry(&old_entry))
				.collect()
		})
		.map_err(|kind| TransformError {
			kind: kind.into(),
			original_entry: old_entry,
		})
}
