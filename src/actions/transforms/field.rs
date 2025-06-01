/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the trait [`TransformField`] as well as all types that implement it
//! And [`Field`] enum that can be used to refer to a [`Message`](`crate::sinks::message::Message`)'s field

pub mod caps;
pub mod extract;
pub mod replace;
pub mod set;
pub mod shorten;
pub mod trim;

pub use self::{
	caps::Caps, extract::Extract, replace::Replace, set::Set, shorten::Shorten, trim::Trim,
};

#[cfg(feature = "action-html-decode")]
pub mod decode_html;
#[cfg(feature = "action-html-decode")]
pub use self::decode_html::DecodeHtml;

use std::{
	convert::Infallible,
	fmt::{self, Debug},
};

use super::{
	Transform,
	result::{OptionUnwrapTransformResultExt, TransformResult, TransformedEntry},
};
use crate::{
	actions::transforms::error::TransformErrorKind,
	entry::{Entry, EntryId},
	maybe_send::MaybeSendSync,
};

/// Transform/change the value of a field of an [`Entry `]
pub trait TransformField: MaybeSendSync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<TransformErrorKind>;

	/// Transform/change the `field` into a new one or `None` specifying what happens if `None` is returned
	///
	/// # Errors
	/// Refer to implementator's docs. Most of them never error but some do
	// TODO: make async
	fn transform_field(
		&mut self,
		old_val: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err>;
}

/// List of all available fields for transformations
#[derive(Clone, Copy, Debug)]
pub enum Field {
	/// [`Message::title`](`crate::sinks::message::Message::title`) field
	Title,
	/// [`Message::body`](`crate::sinks::message::Message::body`) field
	Body,
	/// [`Message::link`](`crate::sinks::message::Message::link`) field
	Link,
	/// [`Entry::id`](`crate::entry::Entry::id`) field
	Id,
	/// [`Entry::reply_to`](`crate::entry::Entry::reply_to`) field
	ReplyTo,
	/// [`Entry::raw_contents`](`crate::entry::Entry::raw_contents`) field
	RawContents,
}

/// Adapt [`TransformField`] to implement [`Transform`] by running [`TransformField::transform_field`] on the specified field.
#[derive(Clone, Debug)]
pub struct TransformFieldAdapter<T>
where
	T: TransformField,
{
	/// The field to transform/change
	pub field: Field,

	/// The transformator that's going to decide what the new value of the field should be
	pub transformator: T,
}

impl<T> Transform for TransformFieldAdapter<T>
where
	T: TransformField,
{
	type Err = TransformErrorKind;

	async fn transform_entry(&mut self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
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
				new_entry.msg.link = self
					.transformator
					.transform_field(entry.msg.link.as_deref())
					.map_err(Into::into)?;
			}
			Field::Id => {
				new_entry.id = self
					.transformator
					.transform_field(entry.id.as_deref())
					.map_err(Into::into)?
					.and_then(|id| EntryId::new(id).unwrap_or_empty());
			}
			Field::ReplyTo => {
				new_entry.reply_to = self
					.transformator
					.transform_field(entry.reply_to.as_deref())
					.map_err(Into::into)?
					.and_then(|id| EntryId::new(id).unwrap_or_empty());
			}
			Field::RawContents => {
				new_entry.raw_contents = self
					.transformator
					.transform_field(entry.msg.body.as_deref())
					.map_err(Into::into)?;
			}
		}

		Ok(vec![new_entry])
	}
}

impl TransformField for () {
	type Err = Infallible;

	fn transform_field(
		&mut self,
		_old_val: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		Ok(TransformResult::default())
	}
}

impl TransformField for Infallible {
	type Err = Infallible;

	fn transform_field(
		&mut self,
		_old_val: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl TransformField for ! {
	type Err = !;

	fn transform_field(
		&mut self,
		_old_val: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		match *self {}
	}
}

impl<T> TransformField for Option<T>
where
	T: TransformField,
{
	type Err = T::Err;

	fn transform_field(
		&mut self,
		old_val: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		let Some(inner) = self else {
			return Ok(TransformResult::default());
		};

		inner.transform_field(old_val)
	}
}

impl<T> TransformField for &mut T
where
	T: TransformField,
{
	type Err = T::Err;

	fn transform_field(
		&mut self,
		old_val: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		(*self).transform_field(old_val)
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
			Self::RawContents => "Entry::raw_contents",
		};

		f.write_str(name)
	}
}
