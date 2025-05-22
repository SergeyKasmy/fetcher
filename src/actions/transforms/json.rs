/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Json`] parser

pub mod error;

pub use self::error::JsonError;

use self::error::{ErrorLocation, JsonErrorInner};
use super::Transform;
use crate::{
	StaticStr,
	actions::transforms::{
		error::RawContentsNotSetError,
		result::{OptionUnwrapTransformResultExt, TransformedEntry, TransformedMessage},
	},
	entry::Entry,
	sinks::message::Media,
	utils::OptionExt,
};

use either::Either;
use serde_json::Value;

// TODO: migrate to serde_json::Value::pointer() API instead
/// JSON parser
#[derive(Debug)]
pub struct Json {
	/// Query to find an item/entry/article in the list
	pub item: Option<Pointer>,
	/// Query to find the title of an item
	pub title: Option<Query>,
	/// One or more query to find the text of an item. If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field
	pub text: Option<Vec<Query>>,
	/// Query to find the id of an item
	pub id: Option<Query>,
	/// Query to find the link to an item
	pub link: Option<Query>,
	/// Query to find the image of that item
	pub img: Option<Vec<Query>>,
}

#[derive(Clone, Debug)]
pub struct Pointer(pub StaticStr);

/// A query to get the value of a JSON field
#[derive(Debug)]
pub struct Query {
	/// a pointer to the JSON key
	pub pointer: Pointer,

	/// whether this query is fine to be ignored if not found
	pub optional: bool,
}

impl Transform for Json {
	type Err = JsonError;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let json: Value =
			serde_json::from_str(entry.raw_contents.as_ref().ok_or(RawContentsNotSetError)?)?;

		let items = match self.item.as_ref() {
			Some(pointer) => match json.pointer(&pointer.0) {
				Some(items) => items,
				None => {
					return Err(JsonError::Inner {
						error: JsonErrorInner::KeyNotFound {
							pointer: pointer.clone(),
						},
						r#where: ErrorLocation::Item,
					});
				}
			},
			// use JSON root if item query is not set
			None => &json,
		};

		let items = if let Some(items) = items.as_array() {
			Either::Left(items.iter())
		} else if let Some(items) = items.as_object() {
			// ignore map keys, iterate over values only
			Either::Right(items.iter().map(|(_, v)| v))
		} else {
			return Err(JsonError::Inner {
				r#where: ErrorLocation::Item,
				error: JsonErrorInner::KeyWrongType {
					pointer: self
						.item
						.as_ref()
						.map_or_else(|| Pointer("/".into()), |ptr| ptr.clone()),
					expected_type: "iterator (array, map)",
					found_type: format!("{items:?}"),
				},
			});
		};

		items
			.into_iter()
			.map(|item| self.extract_entry(item))
			.collect::<Result<Vec<_>, _>>()
	}
}

impl Pointer {
	pub fn new<T: Into<StaticStr>>(ptr: T) -> Self {
		Self(ptr.into())
	}
}

impl Json {
	fn extract_entry(&self, item: &Value) -> Result<TransformedEntry, JsonError> {
		let title = self
			.title
			.as_ref()
			.try_and_then(|q| extract_string(item, q))
			.map_err(|error| JsonError::Inner {
				r#where: ErrorLocation::Title,
				error,
			})?;

		let body = self
			.text
			.as_ref()
			.try_and_then(|v| extract_body(item, v))
			.map_err(|(error, index)| JsonError::Inner {
				r#where: ErrorLocation::Text { index },
				error,
			})?;

		let id = self
			.id
			.as_ref()
			.try_and_then(|q| extract_id(item, q))
			.map_err(|error| JsonError::Inner {
				r#where: ErrorLocation::Id,
				error,
			})?;

		let link = self
			.link
			.as_ref()
			.try_and_then(|q| extract_string(item, q))
			.map_err(|error| JsonError::Inner {
				r#where: ErrorLocation::Link,
				error,
			})?;

		let img = self
			.img
			.as_ref()
			.try_map(|v| {
				v.iter()
					.filter_map(|q| extract_string(item, q).transpose())
					.collect::<Result<Vec<_>, _>>()
			})
			.map_err(|error| JsonError::Inner {
				r#where: ErrorLocation::Img,
				error,
			})?;

		// make it none if it's empty
		let img = if let Some(&[]) = img.as_deref() {
			None
		} else {
			img
		};

		Ok(TransformedEntry {
			id: id.map(Into::into).unwrap_or_prev(),
			raw_contents: body.clone().unwrap_or_prev(),
			msg: TransformedMessage {
				title: title.unwrap_or_prev(),
				body: body.unwrap_or_prev(),
				link: link.unwrap_or_prev(),
				media: img
					.map(|v| v.into_iter().map(Media::Photo).collect())
					.unwrap_or_prev(),
			},
			..Default::default()
		})
	}
}

fn extract_value<'a>(item: &'a Value, query: &Query) -> Result<Option<&'a Value>, JsonErrorInner> {
	match item.pointer(&query.pointer.0) {
		Some(v) => Ok(Some(v)),
		None if query.optional => return Ok(None),
		None => {
			return Err(JsonErrorInner::KeyNotFound {
				pointer: query.pointer.clone(),
			});
		}
	}
}

fn extract_string(item: &Value, query: &Query) -> Result<Option<String>, JsonErrorInner> {
	let Some(value) = extract_value(item, query)? else {
		return Ok(None);
	};

	let s = value.as_str().ok_or_else(|| JsonErrorInner::KeyWrongType {
		pointer: query.pointer.clone(),
		expected_type: "string",
		found_type: format!("{value:?}"),
	})?;

	Ok(Some(s.trim().to_owned()))
}

fn extract_body(item: &Value, query: &[Query]) -> Result<Option<String>, (JsonErrorInner, usize)> {
	let body = query
		.iter()
		.enumerate()
		.filter_map(|(idx, query)| {
			extract_string(item, query)
				.map_err(|e| (e, idx))
				.transpose()
		})
		.collect::<Result<Vec<String>, (JsonErrorInner, usize)>>()?
		.join("\n\n");

	if body.is_empty() {
		Ok(None)
	} else {
		Ok(Some(body))
	}
}

fn extract_id(item: &Value, query: &Query) -> Result<Option<String>, JsonErrorInner> {
	let Some(id_val) = extract_value(item, query)? else {
		return Ok(None);
	};

	let id = if let Some(id) = id_val.as_str() {
		id.to_owned()
	} else if let Some(id) = id_val.as_i64() {
		id.to_string()
	} else if let Some(id) = id_val.as_u64() {
		id.to_string()
	} else {
		return Err(JsonErrorInner::KeyWrongType {
			pointer: query.pointer.clone(),
			expected_type: "string/i64/u64",
			found_type: format!("{id_val:?}"),
		});
	};

	Ok(Some(id))
}
