/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Transform`] and [`TransformField`](`field::TransformField`) traits as well as all types that implement it

pub mod print;
pub mod use_as;

pub mod field;
pub mod result;

pub mod error;

pub(crate) mod async_fn;

pub use self::{
	field::{TransformField, caps::Caps, set::Set, shorten::Shorten, trim::Trim},
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

use self::{
	error::{TransformError, TransformErrorKind},
	result::TransformedEntry,
};
use super::{Action, ActionContext, ActionResult};
use crate::{
	actres_try,
	entry::Entry,
	external_save::ExternalSave,
	maybe_send::{MaybeSend, MaybeSendSync},
	sources::Source,
};

use std::convert::Infallible;

/// Adapter of [`Action`] tailored for "transforming"/changing/modifying entries as a whole,
/// or a field of an entry (via [`TransformField`]) in some way.
///
/// A [`Transform`] might return more than one [`Entry`], not only modify existing ones.
///
/// For example, [`Html`] parses a single [`Entry`] containing raw HTML of a web page
/// and transforms it into one or more articles/entries parsed from the page.
pub trait Transform: MaybeSendSync {
	/// Error that may be returned. Returns [`Infallible`] if it never errors
	type Err: Into<TransformErrorKind>;

	/// Transform the [`Entry`] into one or multiple entries.
	///
	/// [`TransformedEntry`] allows to specify exact changes to the source entries
	/// and not have to worry about copying over source entry's unmodified fields
	/// that haven't been changed as this will be done automatically.
	///
	/// # Errors
	/// Refer to implementator's docs.
	fn transform_entry(
		&mut self,
		entry: Entry,
	) -> impl Future<Output = Result<Vec<TransformedEntry>, Self::Err>> + MaybeSend;
}

/// Adapt a [`Transform`] to implement [`Action`] by applying [`Transform::transform_entry`] to each entry
#[derive(Clone, Debug)]
pub struct TransformAction<T>(pub T);

impl Transform for () {
	type Err = Infallible;

	async fn transform_entry(&mut self, _entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		Ok(vec![TransformedEntry::default()])
	}
}

impl Transform for Infallible {
	type Err = Infallible;

	async fn transform_entry(&mut self, _entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl Transform for ! {
	type Err = !;

	async fn transform_entry(&mut self, _entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		match *self {}
	}
}

impl<T> Transform for Option<T>
where
	T: Transform,
{
	type Err = T::Err;

	async fn transform_entry(&mut self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let Some(inner) = self else {
			return Ok(vec![TransformedEntry::default()]);
		};

		inner.transform_entry(entry).await
	}
}

impl<T> Transform for &mut T
where
	T: Transform,
{
	type Err = T::Err;

	fn transform_entry(
		&mut self,
		entry: Entry,
	) -> impl Future<Output = Result<Vec<TransformedEntry>, Self::Err>> + MaybeSend {
		(*self).transform_entry(entry)
	}
}

impl<T> Action for TransformAction<T>
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
			let entries =
				actres_try!(transform_old_entry_into_new_entries(&mut self.0, entry).await);
			transformed_entries.extend(entries);
		}

		ActionResult::Ok(transformed_entries)
	}
}

async fn transform_old_entry_into_new_entries<T>(
	this: &mut T,
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
