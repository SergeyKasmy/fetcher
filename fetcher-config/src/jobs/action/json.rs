/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::Error;
use fetcher_core::{
	action::transform::{
		entry::json::{self as c_json, Json as CJson},
		field::Replace as CReplace,
	},
	utils::OptionExt,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Json {
	pub item: Option<Query>,
	pub title: Option<StringQuery>,
	pub text: Option<Vec<StringQuery>>,
	pub id: Option<StringQuery>,
	pub link: Option<StringQuery>,
	pub img: Option<Vec<StringQuery>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Key {
	String(String),
	Usize(usize),
}
pub type Keys = Vec<Key>;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Query {
	#[serde(rename = "query")]
	pub keys: Keys,
	// TODO: should itemq really be allowed to be marked as optional?
	pub optional: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct StringQuery {
	#[serde(flatten)]
	pub query: Query,
	pub regex: Option<JsonQueryRegex>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct JsonQueryRegex {
	re: String,
	replace_with: String,
}

impl Json {
	pub fn parse(self) -> Result<CJson, Error> {
		Ok(CJson {
			item: self.item.map(Query::parse),
			title: self.title.try_map(StringQuery::parse)?,

			text: self.text.try_map(|v| {
				v.into_iter()
					.map(StringQuery::parse)
					.collect::<Result<_, _>>()
			})?,

			id: self.id.try_map(StringQuery::parse)?,
			link: self.link.try_map(StringQuery::parse)?,

			img: self.img.try_map(|v| {
				v.into_iter()
					.map(StringQuery::parse)
					.collect::<Result<_, _>>()
			})?,
		})
	}
}

impl Key {
	#[must_use]
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
	pub fn parse(self) -> Result<CReplace, Error> {
		CReplace::new(&self.re, self.replace_with).map_err(Into::into)
	}
}
