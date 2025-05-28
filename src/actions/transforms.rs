/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Transform`] and [`TransformField`](`field::TransformField`) traits as well as all types that implement it

pub mod async_fn;
pub mod print;
pub mod use_as;

pub mod field;
pub mod result;

pub mod error;

use std::convert::Infallible;

pub use self::{
	field::{caps::Caps, set::Set, shorten::Shorten, trim::Trim},
	print::DebugPrint,
	use_as::Use,
};

#[cfg(feature = "action-http")]
pub mod http;
#[cfg(feature = "action-http")]
pub use self::http::Http;

#[cfg(feature = "action-feed")]
pub mod feed;
#[cfg(feature = "action-feed")]
pub use self::feed::Feed;

#[cfg(feature = "action-json")]
pub mod json;
#[cfg(feature = "action-json")]
pub use self::json::Json;

#[cfg(feature = "action-html")]
pub mod html;
#[cfg(feature = "action-html")]
pub use self::html::Html;

use self::error::TransformError;
use self::error::TransformErrorKind;
use self::result::TransformedEntry;
use crate::{
	actres_try,
	entry::Entry,
	external_save::ExternalSave,
	maybe_send::{MaybeSend, MaybeSendSync},
	sources::Source,
};

use super::{Action, ActionContext, ActionResult};

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
	type Err = TransformError;

	async fn apply<S, E>(
		&mut self,
		entries: Vec<Entry>,
		_ctx: ActionContext<'_, S, E>,
	) -> ActionResult<Self::Err>
	where
		S: Source,
		E: ExternalSave,
	{
		let mut transformed_entries = Vec::new();

		for entry in entries {
			let entries = actres_try!(transform_old_entry_into_new_entries(&self.0, entry).await);
			transformed_entries.extend(entries);
		}

		ActionResult::Ok(transformed_entries)
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

impl Transform for () {
	type Err = Infallible;

	async fn transform_entry(&self, _entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		Ok(vec![TransformedEntry::default()])
	}
}

impl Transform for Infallible {
	type Err = Infallible;

	async fn transform_entry(&self, _entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl Transform for ! {
	type Err = !;

	async fn transform_entry(&self, _entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		match *self {}
	}
}

impl<T> Transform for Option<T>
where
	T: Transform,
{
	type Err = T::Err;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let Some(inner) = self else {
			return Ok(vec![TransformedEntry::default()]);
		};

		inner.transform_entry(entry).await
	}
}
