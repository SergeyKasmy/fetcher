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
	pub itemq: Vec<String>,

	#[serde(rename = "title_query")]
	pub titleq: Option<String>,

	#[serde(rename = "text_query")]
	pub textq: Option<Vec<Query>>,

	#[serde(rename = "id_query")]
	pub idq: String,

	#[serde(rename = "link_query")]
	pub linkq: Option<Query>,

	#[serde(rename = "img_query")]
	pub imgq: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Query {
	pub string: String,
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
			titleq: self.titleq,
			textq: self
				.textq
				.map(|v| v.into_iter().map(Query::parse).collect::<Result<_, _>>())
				.transpose()?,
			idq: self.idq,
			linkq: self.linkq.map(Query::parse).transpose()?,
			imgq: self.imgq,
		})
	}
}

impl Query {
	pub fn parse(self) -> Result<c_json::Query, Error> {
		Ok(c_json::Query {
			string: self.string,
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
