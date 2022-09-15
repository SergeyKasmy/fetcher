/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::Error;
use fetcher_core::action::regex as c_regex;
use fetcher_core::action::transform::entry::html::query as c_query;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum QueryKind {
	Tag(String),
	Class(String),
	Attr { name: String, value: String },
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum DataLocation {
	Text,
	Attr(String),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Query {
	#[serde(flatten)]
	pub kind: QueryKind,
	pub ignore: Option<Vec<QueryKind>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HtmlQueryRegex {
	re: String,
	replace_with: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryData {
	pub query: Vec<Query>,
	pub data_location: DataLocation,
	pub regex: Option<HtmlQueryRegex>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ImageQuery {
	optional: Option<bool>,
	#[serde(flatten)]
	inner: QueryData,
}

impl QueryKind {
	pub fn parse(self) -> c_query::QueryKind {
		use QueryKind::{Attr, Class, Tag};

		match self {
			Tag(val) => c_query::QueryKind::Tag(val),
			Class(val) => c_query::QueryKind::Class(val),
			Attr { name, value } => c_query::QueryKind::Attr { name, value },
		}
	}
}

impl DataLocation {
	fn parse(self) -> c_query::DataLocation {
		use DataLocation::{Attr, Text};

		match self {
			Text => c_query::DataLocation::Text,
			Attr(v) => c_query::DataLocation::Attr(v),
		}
	}
}

impl Query {
	pub fn parse(self) -> c_query::Query {
		c_query::Query {
			kind: self.kind.parse(),
			ignore: self
				.ignore
				.map(|v| v.into_iter().map(QueryKind::parse).collect::<Vec<_>>()),
		}
	}
}

impl HtmlQueryRegex {
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

impl QueryData {
	pub fn parse(self) -> Result<c_query::QueryData, Error> {
		c_query::QueryData::new(
			self.query.into_iter().map(Query::parse).collect(),
			self.data_location.parse(),
			self.regex.map(HtmlQueryRegex::parse).transpose()?,
		)
		.map_err(Into::into)
	}
}

impl ImageQuery {
	pub fn parse(self) -> Result<c_query::ImageQuery, Error> {
		Ok(c_query::ImageQuery {
			optional: self.optional.unwrap_or(false),
			inner: self.inner.parse()?,
		})
	}
}
