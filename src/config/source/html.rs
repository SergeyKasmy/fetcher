/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;
use url::Url;

use crate::source;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum QueryKind {
	Tag { value: String },
	Class { value: String },
	Attr { name: String, value: String },
}

impl QueryKind {
	fn parse(self) -> source::html::QueryKind {
		use QueryKind::*;

		match self {
			Tag { value } => source::html::QueryKind::Tag { value },
			Class { value } => source::html::QueryKind::Class { value },
			Attr { name, value } => source::html::QueryKind::Attr { name, value },
		}
	}
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum DataLocation {
	Text,
	Attr { value: String },
}

impl DataLocation {
	fn parse(self) -> source::html::DataLocation {
		use DataLocation::*;

		match self {
			Text => source::html::DataLocation::Text,
			Attr { value } => source::html::DataLocation::Attr { value },
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct Query {
	kind: Vec<QueryKind>,
	data_location: DataLocation,
}

impl Query {
	fn parse(self) -> source::html::Query {
		source::html::Query {
			kind: self.kind.into_iter().map(|x| x.parse()).collect(),
			data_location: self.data_location.parse(),
		}
	}
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum IdQueryKind {
	String,
	Date,
}

impl IdQueryKind {
	fn parse(self) -> source::html::IdQueryKind {
		match self {
			IdQueryKind::String => source::html::IdQueryKind::String,
			IdQueryKind::Date => source::html::IdQueryKind::Date,
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct IdQuery {
	kind: IdQueryKind,
	#[serde(rename = "query")]
	inner: Query,
}

impl IdQuery {
	fn parse(self) -> source::html::IdQuery {
		source::html::IdQuery {
			kind: self.kind.parse(),
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct LinkQuery {
	prepend: Option<String>,
	#[serde(flatten)]
	inner: Query,
}

impl LinkQuery {
	fn parse(self) -> source::html::LinkQuery {
		source::html::LinkQuery {
			prepend: self.prepend,
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct ImageQuery {
	optional: Option<bool>,
	#[serde(flatten)]
	inner: LinkQuery,
}

impl ImageQuery {
	fn parse(self) -> source::html::ImageQuery {
		source::html::ImageQuery {
			optional: self.optional.unwrap_or(false),
			inner: self.inner.parse(),
		}
	}
}

#[derive(Deserialize, Debug)]
pub(crate) struct Html {
	url: Url,
	#[serde(rename = "item_query")]
	itemq: Vec<QueryKind>,

	#[serde(rename = "text_query")]
	textq: Vec<Query>,

	#[serde(rename = "id_query")]
	idq: IdQuery,

	#[serde(rename = "link_query")]
	linkq: LinkQuery,

	#[serde(rename = "img_query")]
	imgq: Option<ImageQuery>,
}

impl Html {
	pub(crate) fn parse(self) -> source::Html {
		source::Html {
			url: self.url,
			itemq: self.itemq.into_iter().map(|x| x.parse()).collect(),
			textq: self.textq.into_iter().map(|x| x.parse()).collect(),
			idq: self.idq.parse(),
			linkq: self.linkq.parse(),
			imgq: self.imgq.map(|x| x.parse()),
		}
	}
}
