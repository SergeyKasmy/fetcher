/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use fetcher_core::action::transform;

use crate::error::ConfigError;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum QueryKind {
	Tag(String),
	Class(String),
	Attr { name: String, value: String },
}

impl QueryKind {
	pub fn parse(self) -> transform::html::query::QueryKind {
		use QueryKind::{Attr, Class, Tag};

		match self {
			Tag(val) => transform::html::query::QueryKind::Tag(val),
			Class(val) => transform::html::query::QueryKind::Class(val),
			Attr { name, value } => transform::html::query::QueryKind::Attr { name, value },
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum DataLocation {
	Text,
	Attr(String),
}

impl DataLocation {
	fn parse(self) -> transform::html::query::DataLocation {
		use DataLocation::{Attr, Text};

		match self {
			Text => transform::html::query::DataLocation::Text,
			Attr(v) => transform::html::query::DataLocation::Attr(v),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Query {
	#[serde(flatten)]
	pub kind: QueryKind,
	pub ignore: Option<Vec<QueryKind>>,
}

impl Query {
	pub fn parse(self) -> transform::html::query::Query {
		transform::html::query::Query {
			kind: self.kind.parse(),
			ignore: self
				.ignore
				.map(|v| v.into_iter().map(QueryKind::parse).collect::<Vec<_>>()),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryData {
	pub query: Vec<Query>,
	pub data_location: DataLocation,
	pub regex: Option<String>,
}

impl QueryData {
	fn parse(self) -> Result<transform::html::query::QueryData, ConfigError> {
		transform::html::query::QueryData::new(
			self.query.into_iter().map(Query::parse).collect(),
			self.data_location.parse(),
			self.regex.as_deref(),
		)
		.map_err(Into::into)
	}
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct TitleQuery(pub QueryData);

impl TitleQuery {
	pub fn parse(self) -> Result<transform::html::query::TitleQuery, ConfigError> {
		Ok(transform::html::query::TitleQuery(self.0.parse()?))
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TextQuery {
	pub prepend: Option<String>,
	#[serde(flatten)]
	pub inner: QueryData,
}

impl TextQuery {
	pub fn parse(self) -> Result<transform::html::query::TextQuery, ConfigError> {
		Ok(transform::html::query::TextQuery {
			prepend: self.prepend,
			inner: self.inner.parse()?,
		})
	}
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(rename_all = "snake_case", deny_unknown_fields)]	// TODO: check if deny_unknown_fields can be used here, esp with flatten
#[serde(rename_all = "snake_case")]
pub enum IdQueryKind {
	String,
	Date,
}

impl IdQueryKind {
	fn parse(self) -> transform::html::query::IdQueryKind {
		match self {
			IdQueryKind::String => transform::html::query::IdQueryKind::String,
			IdQueryKind::Date => transform::html::query::IdQueryKind::Date,
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IdQuery {
	pub kind: IdQueryKind,
	#[serde(flatten)]
	pub inner: QueryData,
}

impl IdQuery {
	pub fn parse(self) -> Result<transform::html::query::IdQuery, ConfigError> {
		Ok(transform::html::query::IdQuery {
			kind: self.kind.parse(),
			inner: self.inner.parse()?,
		})
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UrlQuery {
	pub prepend: Option<String>,
	#[serde(flatten)]
	pub inner: QueryData,
}

impl UrlQuery {
	pub fn parse(self) -> Result<transform::html::query::UrlQuery, ConfigError> {
		Ok(transform::html::query::UrlQuery {
			prepend: self.prepend,
			inner: self.inner.parse()?,
		})
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ImageQuery {
	optional: Option<bool>,
	#[serde(flatten)]
	url: UrlQuery,
}

impl ImageQuery {
	pub fn parse(self) -> Result<transform::html::query::ImageQuery, ConfigError> {
		Ok(transform::html::query::ImageQuery {
			optional: self.optional.unwrap_or(false),
			url: self.url.parse()?,
		})
	}
}

// #[cfg(test)]
// mod tests {
// 	use super::*;

// 	#[test]
// 	fn query_kind() {
// 		let q1 = QueryKind::Class("Class".to_owned());
// 		let q2 = QueryKind::Tag("Tag".to_owned());
// 		let q3 = QueryKind::Attr {
// 			name: "Name".to_owned(),
// 			value: "Value".to_owned(),
// 		};

// 		eprintln!("{q1:?}\n{q2:?}\n{q3:?}");

// 		let q1 = serde_yaml::to_string(&q1).unwrap();
// 		let q2 = serde_yaml::to_string(&q2).unwrap();
// 		let q3 = serde_yaml::to_string(&q3).unwrap();

// 		eprintln!("{q1:?}\n{q2:?}\n{q3:?}");
// 	}
// }
