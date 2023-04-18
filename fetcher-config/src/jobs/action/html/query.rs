/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::Error;
use fetcher_core::{
	action::{transform::entry::html::query as c_query, transform::field::Replace as CReplace},
	utils::OptionExt,
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "snake_case")] // deny_unknown_fields not allowed since it's flattened in [`Query`]
pub enum ElementKind {
	Tag(String),
	Class(String),
	#[serde(with = "crate::serde_extentions::tuple")]
	Attr(ElementAttr),
}

#[derive(PartialEq, Clone, Default, Debug)]
pub struct ElementAttr {
	pub name: String,
	pub value: String,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Default, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum DataLocation {
	#[default]
	Text,
	Attr(String),
}

#[derive(Deserialize, Serialize, Clone, Default, Debug)] // deny_unknown_fields not allowed since it uses flatten
pub struct ElementQuery {
	#[serde(flatten)]
	pub kind: ElementKind,
	pub ignore: Option<Vec<ElementKind>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemQuery {
	pub query: Vec<ElementQuery>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug)] // deny_unknown_fields not allowed since it's flattened in [`ElementQuery`]
pub struct ElementDataQuery {
	#[serde(default)]
	pub optional: bool,
	pub query: Vec<ElementQuery>,
	pub data_location: DataLocation,
	pub regex: Option<HtmlQueryRegex>,
}

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct HtmlQueryRegex {
	pub re: String,
	pub replace_with: String,
}

impl ElementKind {
	pub fn parse(self) -> c_query::ElementKind {
		use ElementKind::{Attr, Class, Tag};

		match self {
			Tag(val) => c_query::ElementKind::Tag(val),
			Class(val) => c_query::ElementKind::Class(val),
			Attr(ElementAttr { name, value }) => c_query::ElementKind::Attr { name, value },
		}
	}
}

impl DataLocation {
	pub fn parse(self) -> c_query::DataLocation {
		use DataLocation::{Attr, Text};

		match self {
			Text => c_query::DataLocation::Text,
			Attr(v) => c_query::DataLocation::Attr(v),
		}
	}
}

impl ElementQuery {
	pub fn parse(self) -> c_query::ElementQuery {
		c_query::ElementQuery {
			kind: self.kind.parse(),
			ignore: self
				.ignore
				.map(|v| v.into_iter().map(ElementKind::parse).collect::<Vec<_>>()),
		}
	}
}

impl ElementDataQuery {
	pub fn parse(self) -> Result<c_query::ElementDataQuery, Error> {
		Ok(c_query::ElementDataQuery {
			optional: self.optional,
			query: self.query.into_iter().map(ElementQuery::parse).collect(),
			data_location: self.data_location.parse(),
			regex: self.regex.try_map(HtmlQueryRegex::parse)?,
		})
	}
}

impl HtmlQueryRegex {
	pub fn parse(self) -> Result<CReplace, Error> {
		CReplace::new(&self.re, self.replace_with).map_err(Into::into)
	}
}

impl Default for ElementKind {
	fn default() -> Self {
		Self::Tag("div".to_owned())
	}
}

impl<'a> From<&'a ElementAttr> for (&'a String, &'a String) {
	fn from(ElementAttr { name, value }: &'a ElementAttr) -> Self {
		(name, value)
	}
}

impl From<(String, String)> for ElementAttr {
	fn from((name, value): (String, String)) -> Self {
		Self { name, value }
	}
}
