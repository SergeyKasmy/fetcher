/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::source;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum QueryKind {
	Tag { value: String },
	Class { value: String },
	Attr { name: String, value: String },
}

impl QueryKind {
	pub(crate) fn parse(self) -> source::parser::html::query::QueryKind {
		use QueryKind::{Attr, Class, Tag};

		match self {
			Tag { value } => source::parser::html::query::QueryKind::Tag { value },
			Class { value } => source::parser::html::query::QueryKind::Class { value },
			Attr { name, value } => source::parser::html::query::QueryKind::Attr { name, value },
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum DataLocation {
	Text,
	Attr { value: String },
}

impl DataLocation {
	fn parse(self) -> source::parser::html::query::DataLocation {
		use DataLocation::{Attr, Text};

		match self {
			Text => source::parser::html::query::DataLocation::Text,
			Attr { value } => source::parser::html::query::DataLocation::Attr { value },
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
				.map(|v| v.into_iter().map(QueryKind::parse).collect::<Vec<_>>())
				.unwrap_or_default(),
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
#[serde(rename_all = "snake_case", deny_unknown_fields)]
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
pub(crate) struct LinkQuery {
	pub(crate) prepend: Option<String>,
	#[serde(flatten)]
	pub(crate) inner: QueryData,
}

impl LinkQuery {
	pub(crate) fn parse(self) -> source::parser::html::query::LinkQuery {
		source::parser::html::query::LinkQuery {
			prepend: self.prepend,
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ImageQuery {
	optional: Option<bool>,
	#[serde(flatten)]
	inner: LinkQuery,
}

impl ImageQuery {
	pub(crate) fn parse(self) -> source::parser::html::query::ImageQuery {
		source::parser::html::query::ImageQuery {
			optional: self.optional.unwrap_or(false),
			inner: self.inner.parse(),
		}
	}
}
