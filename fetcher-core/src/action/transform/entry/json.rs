/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Json`] parser

use super::TransformEntry;
use crate::action::transform::result::{
	TransformResult as TrRes, TransformedEntry, TransformedMessage,
};
use crate::entry::Entry;
use crate::error::transform::{InvalidUrlError, JsonError, NothingToTransformError};
use crate::sink::Media;

use serde_json::Value;
use url::Url;

/// JSON parser
#[derive(Debug)]
pub struct Json {
	/// Query to find an item/entry/article in the list
	// TODO: make optional
	pub itemq: Vec<String>,
	/// Query to find the title of an item
	pub titleq: Option<String>,
	/// One or more query to find the text of an item. If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field
	pub textq: Option<Vec<TextQuery>>, // adjecent
	/// Query to find the id of an item
	// TODO: make optional
	pub idq: String,
	/// Query to find the link to an item
	pub linkq: Option<TextQuery>,
	/// Query to find the image of that item
	pub imgq: Option<Vec<String>>, // nested
}

#[allow(missing_docs)]
#[derive(Debug)]
pub struct TextQuery {
	pub string: String,
	// TODO: remove these, use regex instead
	pub prepend: Option<String>,
	pub append: Option<String>,
}

impl TransformEntry for Json {
	type Error = JsonError;

	#[tracing::instrument(skip_all)]
	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		let json: Value =
			serde_json::from_str(entry.raw_contents.as_ref().ok_or(NothingToTransformError)?)?;

		let items = self.itemq.iter().try_fold(&json, |acc, x| {
			acc.get(x.as_str())
				.ok_or_else(|| JsonError::JsonParseKeyNotFound(x.clone()))
		})?;

		let items_iter: Box<dyn Iterator<Item = &Value>> = if let Some(items) = items.as_array() {
			Box::new(items.iter())
		} else if let Some(items) = items.as_object() {
			// ignore map keys, iterate over values only
			Box::new(items.iter().map(|(_, v)| v))
		} else {
			return Err(JsonError::JsonParseKeyWrongType {
				key: self.itemq.last().unwrap().clone(),
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
								let mut text_str = {
									let text_val = item.get(&query.string).ok_or_else(|| {
										JsonError::JsonParseKeyNotFound(query.string.clone())
									})?;

									text_val
										.as_str()
										.ok_or_else(|| JsonError::JsonParseKeyWrongType {
											key: query.string.clone(),
											expected_type: "string",
											found_type: format!("{text_val:?}"),
										})?
										.trim()
										.to_owned()
								};

								if query.prepend.is_some() || query.append.is_some() {
									text_str = format!(
										"{prepend}{original}{append}",
										prepend = query.prepend.as_deref().unwrap_or_default(),
										original = text_str,
										append = query.append.as_deref().unwrap_or_default()
									);
								}

								Ok(text_str)
							})
							.collect::<Result<Vec<String>, JsonError>>()?
							.join("\n\n");

						Ok::<_, JsonError>(s)
					})
					.transpose()?;

				let id = {
					let id_val = item
						.get(&self.idq)
						.ok_or_else(|| JsonError::JsonParseKeyNotFound(self.idq.clone()))?;

					if let Some(id) = id_val.as_str() {
						id.to_owned()
					} else if let Some(id) = id_val.as_i64() {
						id.to_string()
					} else if let Some(id) = id_val.as_u64() {
						id.to_string()
					} else {
						return Err(JsonError::JsonParseKeyWrongType {
							key: self.idq.clone(),
							expected_type: "string/i64/u64",
							found_type: format!("{id_val:?}"),
						});
					}
				};

				let link = self
					.linkq
					.as_ref()
					.map(|linkq| {
						let link_val = item
							.get(&linkq.string)
							.ok_or_else(|| JsonError::JsonParseKeyNotFound(linkq.string.clone()))?;
						let mut link_str = link_val
							.as_str()
							.ok_or_else(|| JsonError::JsonParseKeyWrongType {
								key: linkq.string.clone(),
								expected_type: "string",
								found_type: format!("{link_val:?}"),
							})?
							.to_owned();

						if linkq.prepend.is_some() || linkq.append.is_some() {
							link_str = format!(
								"{prepend}{original}{append}",
								prepend = linkq.prepend.as_deref().unwrap_or_default(),
								original = link_str,
								append = linkq.append.as_deref().unwrap_or_default()
							);
						}

						Url::try_from(link_str.as_str())
							.map_err(|e| JsonError::from(InvalidUrlError(e, link_str)))
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
					id: TrRes::New(Some(id)), // TODO: return Old(id) where id is None if json id query is None
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
