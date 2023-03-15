/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Json`] parser

use super::TransformEntry;
use crate::{
	action::transform::{
		error::RawContentsNotSetError,
		field::Replace,
		result::{TransformResult as TrRes, TransformedEntry, TransformedMessage},
	},
	entry::Entry,
	error::InvalidUrlError,
	sink::message::Media,
	utils::OptionExt,
};

use async_trait::async_trait;
use either::Either;
use serde_json::Value;
use std::{borrow::Cow, ops::ControlFlow};
use url::Url;

/// JSON parser
#[derive(Debug)]
pub struct Json {
	/// Query to find an item/entry/article in the list
	pub itemq: Option<Query>,
	/// Query to find the title of an item
	pub titleq: Option<StringQuery>,
	/// One or more query to find the text of an item. If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field
	pub textq: Option<Vec<StringQuery>>, // adjecent
	/// Query to find the id of an item
	pub idq: Option<StringQuery>,
	/// Query to find the link to an item
	pub linkq: Option<StringQuery>,
	/// Query to find the image of that item
	pub imgq: Option<Vec<StringQuery>>, // nested
}

/// JSON key
#[derive(Clone, Debug)]
pub enum Key {
	/// object property
	String(String),
	/// array index
	Usize(usize),
}
/// JSON keys/array of [`Key`] alias for improved readability
pub type Keys = Vec<Key>;

/// All data needed to query, extract, and finalize a string from JSON
#[derive(Debug)]
pub struct StringQuery {
	/// a query to the key to get the string from
	pub query: Query,
	/// a regex to finalize the string
	pub regex: Option<Replace>,
}

/// A query to get the value of a JSON field
#[derive(Debug)]
pub struct Query {
	/// a chain of JSON keys that are needed to be traversed to get to this key
	pub keys: Keys,
	/// whether this query is fine to be ignored if not found
	pub optional: bool,
}

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum JsonError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error("Invalid JSON")]
	Invalid(#[from] serde_json::error::Error),

	#[error("JSON key #{num} not found. From query list: {key_list:?}")]
	KeyNotFound { num: usize, key_list: Keys },

	#[error("JSON key {key:?} wrong type: expected {expected_type}, found {found_type}")]
	KeyWrongType {
		key: Keys,
		expected_type: &'static str,
		found_type: String,
	},

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),
}

#[async_trait]
impl TransformEntry for Json {
	type Err = JsonError;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let json: Value =
			serde_json::from_str(entry.raw_contents.as_ref().ok_or(RawContentsNotSetError)?)?;

		let items = match self.itemq.as_ref() {
			Some(query) => match extract_data(&json, query)? {
				Some(items) => items,
				// don't continue if the items query is optional and wasn't found
				None => return Ok(Vec::new()),
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
			return Err(JsonError::KeyWrongType {
				key: self
					.itemq
					.as_ref()
					.map_or_else(Vec::new, |v| v.keys.clone()),
				expected_type: "iterator (array, map)",
				found_type: format!("{items:?}"),
			});
		};

		items
			.into_iter()
			.map(|item| self.extract_entry(item))
			.collect::<Result<Vec<_>, _>>()
	}
}

impl Json {
	fn extract_entry(&self, item: &Value) -> Result<TransformedEntry, JsonError> {
		let title = self
			.titleq
			.as_ref()
			.try_and_then(|q| extract_string(item, q))?;
		let body = self
			.textq
			.as_ref()
			.try_and_then(|v| extract_body(item, v))?;
		let id = self.idq.as_ref().try_and_then(|q| extract_id(item, q))?;
		let link = self.linkq.as_ref().try_and_then(|q| extract_url(item, q))?;

		let img = self.imgq.as_ref().try_map(|v| {
			v.iter()
				.filter_map(|q| extract_url(item, q).transpose())
				.collect::<Result<Vec<_>, _>>()
		})?;

		Ok(TransformedEntry {
			id: TrRes::Old(id.map(Into::into)),
			raw_contents: TrRes::Old(body.clone()),
			msg: TransformedMessage {
				title: TrRes::Old(title),
				body: TrRes::Old(body),
				link: TrRes::Old(link),
				media: TrRes::Old(img.map(|v| v.into_iter().map(Media::Photo).collect())),
			},
			..Default::default()
		})
	}
}

fn extract_data<'a>(json: &'a Value, query: &Query) -> Result<Option<&'a Value>, JsonError> {
	let data = query.keys.iter().enumerate().try_fold(json, |val, (i, q)| {
		let res_val = match q {
			Key::String(s) => val.get(s),
			Key::Usize(u) => val.get(u),
		};

		match res_val {
			Some(v) => ControlFlow::Continue(v),
			None => ControlFlow::Break(i),
		}
	});

	let data = match data {
		ControlFlow::Continue(v) => v,
		ControlFlow::Break(_) if query.optional => return Ok(None),
		ControlFlow::Break(key) => {
			return Err(JsonError::KeyNotFound {
				num: key,
				key_list: query.keys.clone(),
			})
		}
	};

	Ok(Some(data))
}

fn extract_string(item: &Value, str_query: &StringQuery) -> Result<Option<String>, JsonError> {
	let data = match extract_data(item, &str_query.query) {
		Ok(Some(v)) => v,
		Ok(None) => return Ok(None),
		Err(e) => return Err(e),
	};

	let s = data.as_str().ok_or_else(|| JsonError::KeyWrongType {
		key: str_query.query.keys.clone(),
		expected_type: "string",
		found_type: format!("{data:?}"),
	})?;

	let s = match str_query.regex.as_ref() {
		Some(r) => r.replace(s),
		None => Cow::Borrowed(s),
	};

	Ok(Some(s.trim().to_owned()))
}

fn extract_body(item: &Value, bodyq: &[StringQuery]) -> Result<Option<String>, JsonError> {
	let body = bodyq
		.iter()
		.filter_map(|query| extract_string(item, query).transpose())
		.collect::<Result<Vec<String>, JsonError>>()?
		.join("\n\n");

	if body.is_empty() {
		Ok(None)
	} else {
		Ok(Some(body))
	}
}

fn extract_id(item: &Value, query: &StringQuery) -> Result<Option<String>, JsonError> {
	let id_val = match extract_data(item, &query.query) {
		Ok(Some(v)) => v,
		Ok(None) => return Ok(None),
		Err(e) => return Err(e),
	};

	let id = if let Some(id) = id_val.as_str() {
		id.to_owned()
	} else if let Some(id) = id_val.as_i64() {
		id.to_string()
	} else if let Some(id) = id_val.as_u64() {
		id.to_string()
	} else {
		return Err(JsonError::KeyWrongType {
			key: query.query.keys.clone(),
			expected_type: "string/i64/u64",
			found_type: format!("{id_val:?}"),
		});
	};

	let id = match query.regex.as_ref() {
		Some(r) => r.replace(&id).into_owned(),
		None => id,
	};

	Ok(Some(id))
}

fn extract_url(item: &Value, query: &StringQuery) -> Result<Option<Url>, JsonError> {
	let url_str = match extract_string(item, query) {
		Ok(Some(v)) => v,
		Ok(None) => return Ok(None),
		Err(e) => return Err(e),
	};

	let url = Url::try_from(url_str.as_str()).map_err(|e| InvalidUrlError(e, url_str))?;

	Ok(Some(url))
}
