/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::Error;
use fetcher_core::action::regex as c_regex;
use fetcher_core::action::transform::entry::json as c_json;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Json {
	#[serde(rename = "item_query")]
	pub itemq: Option<Keys>,

	#[serde(rename = "title_query")]
	pub titleq: Option<StringQuery>,

	#[serde(rename = "text_query")]
	pub textq: Option<Vec<StringQuery>>,

	#[serde(rename = "id_query")]
	pub idq: Option<StringQuery>,

	#[serde(rename = "link_query")]
	pub linkq: Option<StringQuery>,

	#[serde(rename = "img_query")]
	pub imgq: Option<Vec<StringQuery>>,
}

pub type Key = String;
pub type Keys = Vec<String>;

#[derive(Deserialize, Serialize, Debug)]
pub struct StringQuery {
	pub query: Keys,
	pub regex: Option<JsonQueryRegex>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JsonQueryRegex {
	re: String,
	replace_with: String,
}

impl Json {
	pub fn parse(self) -> Result<c_json::Json, Error> {
		Ok(c_json::Json {
			itemq: self.itemq,
			titleq: self.titleq.map(StringQuery::parse).transpose()?,

			textq: self
				.textq
				.map(|v| {
					v.into_iter()
						.map(StringQuery::parse)
						.collect::<Result<_, _>>()
				})
				.transpose()?,

			idq: self.idq.map(StringQuery::parse).transpose()?,
			linkq: self.linkq.map(StringQuery::parse).transpose()?,

			imgq: self
				.imgq
				.map(|v| {
					v.into_iter()
						.map(StringQuery::parse)
						.collect::<Result<_, _>>()
				})
				.transpose()?,
		})
	}
}

impl StringQuery {
	pub fn parse(self) -> Result<c_json::StringQuery, Error> {
		Ok(c_json::StringQuery {
			query: self.query,
			regex: self.regex.map(JsonQueryRegex::parse).transpose()?,
		})
	}
}

impl JsonQueryRegex {
	pub fn parse(self) -> Result<c_regex::Regex<c_regex::action::Replace>, Error> {
		c_regex::Regex::new(
			&self.re,
			c_regex::action::Replace {
				with: self.replace_with,
			},
		)
		.map_err(Into::into)
	}
}
