/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

#[derive(Clone, Debug)]
pub(crate) enum QueryKind {
	Tag(String),
	Class(String),
	Attr { name: String, value: String },
}

#[derive(Debug)]
pub(crate) enum DataLocation {
	Text,
	Attr(String),
}

#[derive(Debug)]
pub(crate) struct Query {
	pub(crate) kind: QueryKind,
	pub(crate) ignore: Option<Vec<QueryKind>>,
}

#[derive(Debug)]
pub(crate) struct QueryData {
	pub(crate) query: Vec<Query>,
	pub(crate) data_location: DataLocation,
}

#[derive(Debug)]
pub(crate) struct TitleQuery(pub(crate) QueryData);

#[derive(Debug)]
pub(crate) struct TextQuery {
	pub(crate) prepend: Option<String>,
	pub(crate) inner: QueryData,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum IdQueryKind {
	String,
	Date,
}

#[derive(Debug)]
pub(crate) struct IdQuery {
	pub(crate) kind: IdQueryKind,
	pub(crate) inner: QueryData,
}

#[derive(Debug)]
pub(crate) struct UrlQuery {
	pub(crate) prepend: Option<String>,
	pub(crate) inner: QueryData,
}

#[derive(Debug)]
pub(crate) struct ImageQuery {
	pub(crate) optional: bool,
	pub(crate) url: UrlQuery,
}
