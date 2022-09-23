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
};

use either::Either;
use serde_json::Value;
use std::borrow::Cow;
use url::Url;

/// JSON parser
#[derive(Debug)]
pub struct Json {
	/// Query to find an item/entry/article in the list
	pub itemq: Option<Vec<String>>,
	/// Query to find the title of an item
	pub titleq: Option<String>,
	/// One or more query to find the text of an item. If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field
	pub textq: Option<Vec<Query>>, // adjecent
	/// Query to find the id of an item
	pub idq: Option<String>,
	/// Query to find the link to an item
	pub linkq: Option<Query>,
	/// Query to find the image of that item
	pub imgq: Option<Vec<String>>, // nested
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct Query {
	pub string: String,
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
			.map(|v| {
				v.iter().try_fold(&json, |acc, x| {
					acc.get(x.as_str())
						.ok_or_else(|| JsonError::JsonParseKeyNotFound(x.clone()))
				})
			})
			.transpose()?
			.unwrap_or(&json);

		let items_iter = if let Some(items) = items.as_array() {
			Either::Left(items.iter())
		} else if let Some(items) = items.as_object() {
			// ignore map keys, iterate over values only
			Either::Right(items.iter().map(|(_, v)| v))
		} else {
			return Err(JsonError::JsonParseKeyWrongType {
				key: self
					.itemq
					.as_ref()
					.map_or_else(|| "Root".to_owned(), |v| v.last().unwrap().clone()),
				expected_type: "iterator (array, map)",
				found_type: format!("{items:?}"),
			});
		};

		items_iter
			.into_iter()
			.map(|item| {
				let title = self
					.titleq
					.as_ref()
					.and_then(|s| item.get(s))
					.and_then(serde_json::Value::as_str)
					.map(|s| s.trim().to_owned());

				let body = self
					.textq
					.as_ref()
					.map(|v| {
						let s = v
							.iter()
							.map(|query| {
								let val = item.get(&query.string).ok_or_else(|| {
									JsonError::JsonParseKeyNotFound(query.string.clone())
								})?;

								let s = val
									.as_str()
									.ok_or_else(|| JsonError::JsonParseKeyWrongType {
										key: query.string.clone(),
										expected_type: "string",
										found_type: format!("{val:?}"),
									})?
									.trim();

								let s = match &query.regex {
									Some(r) => r.replace(s).into_owned(),
									None => s.to_owned(),
								};

								Ok(s)
							})
							.collect::<Result<Vec<String>, JsonError>>()?
							.join("\n\n");

						Ok::<_, JsonError>(s)
					})
					.transpose()?;

				let id = self
					.idq
					.as_ref()
					.map(|idq| {
						let id_val = item
							.get(idq)
							.ok_or_else(|| JsonError::JsonParseKeyNotFound(idq.clone()))?;

						let res = if let Some(id) = id_val.as_str() {
							id.to_owned()
						} else if let Some(id) = id_val.as_i64() {
							id.to_string()
						} else if let Some(id) = id_val.as_u64() {
							id.to_string()
						} else {
							return Err(JsonError::JsonParseKeyWrongType {
								key: idq.clone(),
								expected_type: "string/i64/u64",
								found_type: format!("{id_val:?}"),
							});
						};

						Ok(res)
					})
					.transpose()?;

				let link = self
					.linkq
					.as_ref()
					.map(|linkq| {
						let val = item
							.get(&linkq.string)
							.ok_or_else(|| JsonError::JsonParseKeyNotFound(linkq.string.clone()))?;

						let s = val
							.as_str()
							.ok_or_else(|| JsonError::JsonParseKeyWrongType {
								key: linkq.string.clone(),
								expected_type: "string",
								found_type: format!("{val:?}"),
							})?;

						let s = match &linkq.regex {
							Some(r) => r.replace(s),
							None => Cow::Borrowed(s),
						};

						Url::try_from(&*s)
							.map_err(|e| JsonError::from(InvalidUrlError(e, s.into_owned())))
					})
					.transpose()?;

				let img: Option<Url> = self
					.imgq
					.as_ref()
					.map(|imgq| {
						let first = item
							.get(&imgq[0])
							.ok_or_else(|| JsonError::JsonParseKeyNotFound(imgq[0].clone()))?;

						let img_val = imgq.iter().skip(1).try_fold(first, |val, x| {
							val.get(x)
								.ok_or_else(|| JsonError::JsonParseKeyNotFound(x.clone()))
						})?;

						let img_str = img_val
							.as_str()
							.ok_or_else(|| JsonError::JsonParseKeyWrongType {
								key: imgq.last().unwrap().clone(),
								expected_type: "string",
								found_type: format!("{img_val:?}"),
							})?
							.to_owned();

						Url::try_from(img_str.as_str())
							.map_err(|e| JsonError::from(InvalidUrlError(e, img_str)))
					})
					.transpose()?;

				Ok(TransformedEntry {
					id: TrRes::Old(id),
					raw_contents: TrRes::Old(body.clone()),
					msg: TransformedMessage {
						title: TrRes::Old(title),
						body: TrRes::Old(body),
						link: TrRes::Old(link),
						media: TrRes::Old(img.map(|url| vec![Media::Photo(url)])),
					},
				})
			})
			.collect()
	}
}
