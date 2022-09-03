/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::transform::{HtmlError, RegexError};

use regex::Regex;

#[derive(Clone, Debug)]
pub enum QueryKind {
	Tag(String),
	Class(String),
	Attr { name: String, value: String },
}

#[derive(Debug)]
pub enum DataLocation {
	Text,
	Attr(String),
}

#[derive(Debug)]
pub struct Query {
	pub kind: QueryKind,
	pub ignore: Option<Vec<QueryKind>>,
}

#[derive(Debug)]
pub struct QueryData {
	pub query: Vec<Query>,
	pub data_location: DataLocation,
	pub regex: Option<Regex>,
}

impl QueryData {
	pub fn new(
		query: Vec<Query>,
		data_location: DataLocation,
		re: Option<&str>,
	) -> Result<Self, HtmlError> {
		Ok(Self {
			query,
			data_location,
			regex: match re {
				Some(re) => Some(Regex::new(re).map_err(RegexError::InvalidPattern)?),
				None => None,
			},
		})
	}
}

#[derive(Debug)]
pub struct TitleQuery(pub QueryData);

#[derive(Debug)]
pub struct TextQuery {
	pub prepend: Option<String>,
	pub inner: QueryData,
}

#[derive(Clone, Copy, Debug)]
pub enum IdQueryKind {
	String,
	Date,
}

#[derive(Debug)]
pub struct IdQuery {
	pub kind: IdQueryKind,
	pub inner: QueryData,
}

#[derive(Debug)]
pub struct UrlQuery {
	pub prepend: Option<String>,
	pub inner: QueryData,
}

#[derive(Debug)]
pub struct ImageQuery {
	pub optional: bool,
	pub url: UrlQuery,
}
