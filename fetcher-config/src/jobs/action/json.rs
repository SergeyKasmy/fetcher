/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::Error;
use fetcher_core::{
	action::{regex as c_regex, transform::entry::json as c_json},
	utils::OptionExt,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Json {
	#[serde(rename = "item_query")]
	pub itemq: Option<Query>,

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Key {
	String(String),
	Usize(usize),
}
pub type Keys = Vec<Key>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Query {
	#[serde(rename = "query")]
	pub keys: Keys,
	pub optional: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct StringQuery {
	#[serde(flatten)]
	pub query: Query,
	pub regex: Option<JsonQueryRegex>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct JsonQueryRegex {
	re: String,
	replace_with: String,
}

impl Json {
	pub fn parse(self) -> Result<c_json::Json, Error> {
		Ok(c_json::Json {
			itemq: self.itemq.map(Query::parse),
			titleq: self.titleq.try_map(StringQuery::parse)?,

			textq: self.textq.try_map(|v| {
				v.into_iter()
					.map(StringQuery::parse)
					.collect::<Result<_, _>>()
			})?,

			idq: self.idq.try_map(StringQuery::parse)?,
			linkq: self.linkq.try_map(StringQuery::parse)?,

			imgq: self.imgq.try_map(|v| {
				v.into_iter()
					.map(StringQuery::parse)
					.collect::<Result<_, _>>()
			})?,
		})
	}
}

impl Key {
	pub fn parse(self) -> c_json::Key {
		match self {
			Key::String(s) => c_json::Key::String(s),
			Key::Usize(u) => c_json::Key::Usize(u),
		}
	}
}

impl Query {
	pub fn parse(self) -> c_json::Query {
		c_json::Query {
			keys: self.keys.into_iter().map(Key::parse).collect(),
			optional: self.optional.unwrap_or(false),
		}
	}
}

impl StringQuery {
	pub fn parse(self) -> Result<c_json::StringQuery, Error> {
		Ok(c_json::StringQuery {
			query: self.query.parse(),
			regex: self.regex.try_map(JsonQueryRegex::parse)?,
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
