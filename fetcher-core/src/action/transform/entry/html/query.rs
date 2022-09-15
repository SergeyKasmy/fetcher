/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
	action::regex::{action::Replace, Regex},
	error::transform::HtmlError,
};

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
	pub regex: Option<Regex<Replace>>,
}

impl QueryData {
	pub fn new(
		query: Vec<Query>,
		data_location: DataLocation,
		regex: Option<Regex<Replace>>,
	) -> Result<Self, HtmlError> {
		Ok(Self {
			query,
			data_location,
			regex,
		})
	}
}

#[derive(Debug)]
pub struct ImageQuery {
	pub optional: bool,
	pub inner: QueryData,
}
