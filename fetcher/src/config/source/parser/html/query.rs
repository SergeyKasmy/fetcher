/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub(crate) enum QueryKind {
	Tag(String),
	Class(String),
	Attr { name: String, value: String },
}

impl QueryKind {
	pub(crate) fn parse(self) -> source::parser::html::query::QueryKind {
		use QueryKind::{Attr, Class, Tag};

		match self {
			Tag(val) => source::parser::html::query::QueryKind::Tag(val),
			Class(val) => source::parser::html::query::QueryKind::Class(val),
			Attr { name, value } => source::parser::html::query::QueryKind::Attr { name, value },
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub(crate) enum DataLocation {
	Text,
	Attr(String),
}

impl DataLocation {
	fn parse(self) -> source::parser::html::query::DataLocation {
		use DataLocation::{Attr, Text};

		match self {
			Text => source::parser::html::query::DataLocation::Text,
			Attr(v) => source::parser::html::query::DataLocation::Attr(v),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Query {
	#[serde(flatten)]
	pub(crate) kind: QueryKind,
	pub(crate) ignore: Option<Vec<QueryKind>>,
}

impl Query {
	pub(crate) fn parse(self) -> source::parser::html::query::Query {
		source::parser::html::query::Query {
			kind: self.kind.parse(),
			ignore: self
				.ignore
				.map(|v| v.into_iter().map(QueryKind::parse).collect::<Vec<_>>()),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct QueryData {
	pub(crate) query: Vec<Query>,
	pub(crate) data_location: DataLocation,
}

impl QueryData {
	fn parse(self) -> source::parser::html::query::QueryData {
		source::parser::html::query::QueryData {
			query: self.query.into_iter().map(Query::parse).collect(),
			data_location: self.data_location.parse(),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub(crate) struct TitleQuery(pub(crate) QueryData);

impl TitleQuery {
	pub(crate) fn parse(self) -> source::parser::html::query::TitleQuery {
		source::parser::html::query::TitleQuery(self.0.parse())
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TextQuery {
	pub(crate) prepend: Option<String>,
	#[serde(flatten)]
	pub(crate) inner: QueryData,
}

impl TextQuery {
	pub(crate) fn parse(self) -> source::parser::html::query::TextQuery {
		source::parser::html::query::TextQuery {
			prepend: self.prepend,
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(rename_all = "snake_case", deny_unknown_fields)]	// TODO: check if deny_unknown_fields can be used here, esp with flatten
#[serde(rename_all = "snake_case")]
pub(crate) enum IdQueryKind {
	String,
	Date,
}

impl IdQueryKind {
	fn parse(self) -> source::parser::html::query::IdQueryKind {
		match self {
			IdQueryKind::String => source::parser::html::query::IdQueryKind::String,
			IdQueryKind::Date => source::parser::html::query::IdQueryKind::Date,
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct IdQuery {
	pub(crate) kind: IdQueryKind,
	#[serde(flatten)]
	pub(crate) inner: QueryData,
}

impl IdQuery {
	pub(crate) fn parse(self) -> source::parser::html::query::IdQuery {
		source::parser::html::query::IdQuery {
			kind: self.kind.parse(),
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct UrlQuery {
	pub(crate) prepend: Option<String>,
	#[serde(flatten)]
	pub(crate) inner: QueryData,
}

impl UrlQuery {
	pub(crate) fn parse(self) -> source::parser::html::query::UrlQuery {
		source::parser::html::query::UrlQuery {
			prepend: self.prepend,
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ImageQuery {
	optional: Option<bool>,
	#[serde(flatten)]
	url: UrlQuery,
}

impl ImageQuery {
	pub(crate) fn parse(self) -> source::parser::html::query::ImageQuery {
		source::parser::html::query::ImageQuery {
			optional: self.optional.unwrap_or(false),
			url: self.url.parse(),
		}
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
