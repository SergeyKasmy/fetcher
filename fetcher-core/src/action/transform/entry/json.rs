/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Json`] parser

use super::TransformEntry;
use crate::{
	action::{
		regex::{action::Replace, Regex},
		transform::result::{TransformResult as TrRes, TransformedEntry, TransformedMessage},
	},
	entry::Entry,
	error::transform::{InvalidUrlError, JsonError, NothingToTransformError},
	sink::Media,
	utils::OptionExt,
};

use either::Either;
use serde_json::Value;
use std::borrow::Cow;
use url::Url;

/// JSON parser
#[derive(Debug)]
pub struct Json {
	/// Query to find an item/entry/article in the list
	pub itemq: Option<Keys>,
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

/// JSON key alias for improved readability
pub type Key = String;
/// JSON keys/array of [`Key`] alias for improved readability
pub type Keys = Vec<String>;

/// All data needed to query, extract, and finalize a string from JSON
#[derive(Debug)]
pub struct StringQuery {
	/// chain of keys that are needed to be traversed to reach the quieried string
	pub query: Keys,
	/// a regex to finalize the string
	pub regex: Option<Regex<Replace>>,
}

impl TransformEntry for Json {
	type Error = JsonError;

	#[tracing::instrument(skip_all)]
	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		let json: Value =
			serde_json::from_str(entry.raw_contents.as_ref().ok_or(NothingToTransformError)?)?;

		let items = self
			.itemq
			.as_ref()
			.try_map(|v| {
				v.iter().try_fold(&json, |acc, x| {
					acc.get(x.as_str()).ok_or_else(|| JsonError::KeyNotFound {
						name: x.clone(),
						key_list: v.clone(),
					})
				})
			})?
			.unwrap_or(&json);

		let items = if let Some(items) = items.as_array() {
			Either::Left(items.iter())
		} else if let Some(items) = items.as_object() {
			// ignore map keys, iterate over values only
			Either::Right(items.iter().map(|(_, v)| v))
		} else {
			return Err(JsonError::KeyWrongType {
				key: self.itemq.as_ref().map_or_else(Vec::new, Clone::clone),
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
		let title = self.titleq.as_ref().try_map(|q| extract_string(item, q))?;
		let body = self.textq.as_ref().try_map(|v| extract_body(item, v))?;
		let id = self.idq.as_ref().try_map(|q| extract_id(item, q))?;
		let link = self.linkq.as_ref().try_map(|q| extract_url(item, q))?;

		let img = self.imgq.as_ref().try_map(|v| {
			v.iter()
				.map(|q| extract_url(item, q))
				.collect::<Result<Vec<_>, _>>()
		})?;

		Ok(TransformedEntry {
			id: TrRes::Old(id),
			raw_contents: TrRes::Old(body.clone()),
			msg: TransformedMessage {
				title: TrRes::Old(title),
				body: TrRes::Old(body),
				link: TrRes::Old(link),
				media: TrRes::Old(img.map(|v| v.into_iter().map(Media::Photo).collect())),
			},
		})
	}
}

fn extract_data<'a>(json: &'a Value, queries: &Keys) -> Result<&'a Value, JsonError> {
	if queries.is_empty() {
		return Ok(json);
	}

	let first = json
		.get(&queries[0])
		.ok_or_else(|| JsonError::KeyNotFound {
			name: queries[0].clone(),
			key_list: queries.clone(),
		})?;

	let data = queries.iter().skip(1).try_fold(first, |val, q| {
		// val.get(q).ok_or_else(|| JsonError::KeyNotFound(q.clone()))
		val.get(q).ok_or_else(|| JsonError::KeyNotFound {
			name: q.clone(),
			key_list: queries.clone(),
		})
	})?;

	Ok(data)
}

fn extract_string(item: &Value, str_query: &StringQuery) -> Result<String, JsonError> {
	let data = extract_data(item, &str_query.query)?;

	let s = data.as_str().ok_or_else(|| JsonError::KeyWrongType {
		key: str_query.query.clone(),
		expected_type: "string",
		found_type: format!("{data:?}"),
	})?;

	let s = match str_query.regex.as_ref() {
		Some(r) => r.replace(s),
		None => Cow::Borrowed(s),
	};

	Ok(s.trim().to_owned())
}

fn extract_body(item: &Value, bodyq: &[StringQuery]) -> Result<String, JsonError> {
	Ok(bodyq
		.iter()
		.map(|query| extract_string(item, query))
		.collect::<Result<Vec<String>, JsonError>>()?
		.join("\n\n"))
}

fn extract_id(item: &Value, query: &StringQuery) -> Result<String, JsonError> {
	let id_val = extract_data(item, &query.query)?;

	let id = if let Some(id) = id_val.as_str() {
		id.to_owned()
	} else if let Some(id) = id_val.as_i64() {
		id.to_string()
	} else if let Some(id) = id_val.as_u64() {
		id.to_string()
	} else {
		return Err(JsonError::KeyWrongType {
			key: query.query.clone(),
			expected_type: "string/i64/u64",
			found_type: format!("{id_val:?}"),
		});
	};

	let id = match query.regex.as_ref() {
		Some(r) => r.replace(&id).into_owned(),
		None => id,
	};

	Ok(id)
}

fn extract_url(item: &Value, query: &StringQuery) -> Result<Url, JsonError> {
	let url_str = extract_string(item, query)?;

	Url::try_from(url_str.as_str()).map_err(|e| JsonError::from(InvalidUrlError(e, url_str)))
}
