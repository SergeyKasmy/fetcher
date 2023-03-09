/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the trait [`TransformField`] as well as all types that implement it
//! And [`Field`] enum that can be used to refer to a [`Message`](`crate::sink::Message`)'s field

pub mod caps;
pub mod set;
pub mod shorten;
pub mod trim;

pub use crate::action::regex;

use async_trait::async_trait;
use std::fmt::Debug;
use url::Url;

use super::{result::TransformResult, Transform};
use crate::{
	action::transform::error::{TransformError, TransformErrorKind},
	entry::Entry,
	error::InvalidUrlError,
	sink::message::Message,
	utils::OptionExt,
};

/// Transform/change the value of a field of an [`Entry `]
pub trait TransformField: Debug + Send + Sync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<TransformErrorKind>;

	/// Transform/change the `field` into a new one or `None` specifying what happens if `None` is returned
	///
	/// # Errors
	/// Refer to implementator's docs. Most of them never error but some do
	fn transform_field(&self, old_val: Option<&str>) -> Result<TransformResult<String>, Self::Err>;
}

// TODO: make a new name
/// A wrapper around a [`TransformField`] that takes a value out of a [`Field`], passes it to the transformator,
/// and processes the result - updating, removing, or retaining the old value of the field as specified by the transformator
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

#[async_trait]
impl<T> Transform for TransformFieldWrapper<T>
where
	T: TransformField,
{
	async fn transform(&self, entry: &Entry) -> Result<Vec<Entry>, TransformError> {
		// TODO: remove this, take entry by ownership?
		let mut entry = entry.clone();
		// old value of the field
		let old_val = match self.field {
			Field::Title => entry.msg.title.take(),
			Field::Body => entry.msg.body.take(),
			Field::Link => entry.msg.link.take().map(|u| u.to_string()),
			Field::Id => entry.id.take().map(|id| id.0),
			Field::ReplyTo => entry.reply_to.take().map(|id| id.0),
			Field::RawContets => entry.raw_contents.take(),
		};

		let new_val = self
			.transformator
			.transform_field(old_val.as_deref())
			.map_err(|kind| TransformError {
				kind: kind.into(),
				original_entry: entry.clone(),
			})?;

		// finalized value of the field. It's the new value that can get replaced with the old value if requested
		let final_val = new_val.get(old_val);

		let new_entry = match self.field {
			Field::Title => Entry {
				msg: Message {
					title: final_val,
					..entry.msg
				},
				..entry
			},
			Field::Body => Entry {
				msg: Message {
					body: final_val,
					..entry.msg
				},
				..entry
			},
			Field::Link => {
				let link = final_val.try_map(|s| {
					Url::try_from(s.as_str()).map_err(|e| TransformError {
						kind: TransformErrorKind::FieldLinkTransformInvalidUrl(InvalidUrlError(
							e, s,
						)),
						original_entry: entry.clone(),
					})
				})?;

				Entry {
					msg: Message { link, ..entry.msg },
					..entry
				}
			}
			Field::Id => Entry {
				id: final_val.map(Into::into),
				..entry
			},
			Field::ReplyTo => Entry {
				reply_to: final_val.map(Into::into),
				..entry
			},
			Field::RawContets => Entry {
				raw_contents: final_val,
				..entry
			},
		};

		Ok(vec![new_entry])
	}
}

/// List of all available fields for transformations
#[derive(Clone, Copy, Debug)]
pub enum Field {
	/// [`Message.title`] field
	Title,
	/// [`Message.body`] field
	Body,
	/// [`Message.link`] field
	Link,
	/// [`Entry.id`] field
	Id,
	/// [`Entry.reply_to`] field
	ReplyTo,
	/// [`Entry.raw_contents`] field
	RawContets,
}
