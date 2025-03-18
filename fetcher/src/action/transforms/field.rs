/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the trait [`TransformField`] as well as all types that implement it
//! And [`Field`] enum that can be used to refer to a [`Message`](Message)'s field
//!
//! [Message]: crate::sink::message::Message

pub mod caps;
pub mod decode_html;
pub mod extract;
pub mod replace;
pub mod set;
pub mod shorten;
pub mod trim;

pub use self::{
	caps::Caps, extract::Extract, replace::Replace, set::Set, shorten::Shorten, trim::Trim,
};

use std::fmt::{self, Debug};
use url::Url;

use super::{
	Transform,
	result::{TransformResult, TransformedEntry},
};
use crate::{action::transforms::error::TransformErrorKind, entry::Entry, error::InvalidUrlError};

/// Transform/change the value of a field of an [`Entry `]
pub trait TransformField: Debug + Send + Sync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<TransformErrorKind>;

	/// Transform/change the `field` into a new one or `None` specifying what happens if `None` is returned
	///
	/// # Errors
	/// Refer to implementator's docs. Most of them never error but some do
	fn transform_field(&self, old_val: Option<&str>) -> Result<TransformResult<String>, Self::Err>;

	fn in_field(self, field: Field) -> TransformFieldWrapper<Self>
	where
		Self: Sized,
	{
		TransformFieldWrapper {
			field,
			transformator: self,
		}
	}

	fn in_body(self) -> TransformFieldWrapper<Self>
	where
		Self: Sized,
	{
		self.in_field(Field::Body)
	}
}

/// List of all available fields for transformations
#[derive(Clone, Copy, Debug)]
pub enum Field {
	/// [`Message::title`] field
	Title,
	/// [`Message::body`] field
	Body,
	/// [`Message::link`] field
	Link,
	/// [`Entry::id`] field
	Id,
	/// [`Entry::reply_to`] field
	ReplyTo,
	/// [`Entry::raw_contents`] field
	RawContets,
}

// TODO: make a new name
/// A wrapper around a [`TransformField`].
///
/// It takes a value out of a [`Field`], passes it to the transformator,
/// and processes the result - updating, removing, or retaining the old value of the field as specified by the transformator
#[doc(hidden)]
#[derive(Debug)]
pub struct TransformFieldWrapper<T>
where
	T: TransformField,
{
	/// The field to transform/change
	pub field: Field,

	/// The transformator that's going to decide what the new value of the field should be
	pub transformator: T,
}

impl<T> Transform for TransformFieldWrapper<T>
where
	T: TransformField,
{
	type Err = TransformErrorKind;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let mut new_entry = TransformedEntry::default();

		match self.field {
			Field::Title => {
				new_entry.msg.title = self
					.transformator
					.transform_field(entry.msg.title.as_deref())
					.map_err(Into::into)?;
			}
			Field::Body => {
				new_entry.msg.body = self
					.transformator
					.transform_field(entry.msg.body.as_deref())
					.map_err(Into::into)?;
			}
			Field::Link => {
				let old_link = entry.msg.link.as_ref().map(|u| u.to_string());

				new_entry.msg.link = self
					.transformator
					.transform_field(old_link.as_deref())
					.map_err(Into::into)?
					.try_map(|s| {
						Url::try_from(s.as_str()).map_err(|e| {
							TransformErrorKind::FieldLinkTransformInvalidUrl(InvalidUrlError(e, s))
						})
					})?;
			}
			Field::Id => {
				new_entry.id = self
					.transformator
					.transform_field(entry.id.as_deref())
					.map_err(Into::into)?
					.map(Into::into);
			}
			Field::ReplyTo => {
				new_entry.reply_to = self
					.transformator
					.transform_field(entry.reply_to.as_deref())
					.map_err(Into::into)?
					.map(Into::into);
			}
			Field::RawContets => {
				new_entry.raw_contents = self
					.transformator
					.transform_field(entry.msg.body.as_deref())
					.map_err(Into::into)?;
			}
		}

		Ok(vec![new_entry])
	}
}

impl fmt::Display for Field {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::Title => "Message::title",
			Self::Body => "Message::body",
			Self::Link => "Message::link",
			Self::Id => "Entry::id",
			Self::ReplyTo => "Entry::reply_to",
			Self::RawContets => "Entry::raw_contents",
		};

		f.write_str(name)
	}
}
