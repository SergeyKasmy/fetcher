/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::Error;
use fetcher_core::{
	action::{transform::entry::html::query as c_query, transform::field::Replace as CReplace},
	utils::OptionExt,
};

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")] // deny_unknown_fields not allowed since it's flattened in [`Query`]
pub enum ElementKind {
	Tag(String),
	Class(String),
	Attr(ElementAttr),
}

#[derive(Clone, Debug)]
pub struct ElementAttr {
	name: String,
	value: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum DataLocation {
	Text,
	Attr(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)] // deny_unknown_fields not allowed since it uses flatten
pub struct ElementQuery {
	#[serde(flatten)]
	pub kind: ElementKind,
	pub ignore: Option<Vec<ElementKind>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct HtmlQueryRegex {
	re: String,
	replace_with: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)] // deny_unknown_fields not allowed since it's flattened in [`ImageQuery`]
pub struct ElementDataQuery {
	pub optional: Option<bool>,
	pub query: Vec<ElementQuery>,
	pub data_location: DataLocation,
	pub regex: Option<HtmlQueryRegex>,
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
			optional: self.optional.unwrap_or(false),
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

impl Serialize for ElementAttr {
	fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let mut map = ser.serialize_map(Some(1))?;
		map.serialize_entry(&self.name, &self.value)?;
		map.end()
	}
}

impl<'de> Deserialize<'de> for ElementAttr {
	fn deserialize<D>(deser: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct ElementAttrVisitor;

		impl<'de> Visitor<'de> for ElementAttrVisitor {
			type Value = ElementAttr;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				f.write_str("a map with a single element with a string key and a string value")
			}

			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::MapAccess<'de>,
			{
				let Some((name, value)) = map.next_entry()? else {
					return Err(serde::de::Error::invalid_length(0, &self));
				};

				Ok(ElementAttr { name, value })
			}
		}

		deser.deserialize_map(ElementAttrVisitor)
	}
}
