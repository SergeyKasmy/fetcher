/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`ElementQuery`], that checks if an HTML element fits all provided requirements,
//! and [`ElementDataQuery`] that extracts some kind of data from the said element

use std::fmt::Display;

use crate::action::transforms::field::Replace;

/// The type of item that should be queried
#[derive(Clone, Debug)]
pub enum ElementKind {
	/// An HTML tag
	Tag(String),
	/// An HTML class
	Class(String),
	/// An HTML attribute
	Attr {
		/// Name of the attr
		name: String,
		/// Value of the attr
		value: String,
	},
}

/// The location of the data in the quiried tag
#[derive(Clone, Debug)]
pub enum DataLocation {
	/// In the text part of the tag
	Text,
	/// In an attribute
	Attr(String),
}

/// A query for an HTML tag
#[derive(Clone, Debug)]
pub struct ElementQuery {
	/// Query the tag should match against
	pub kind: ElementKind,
	/// Query the tag should never match
	pub ignore: Option<Vec<ElementKind>>,
}

/// A query for a complete HTML tag. Traverses all queries one by one and extracts the data from it's [`DataLocation`], optionally transforming the data via regex
/// Example:
/// ```text
/// [`ElementDataQuery`] {
///     query: [Tag("div"), Attr { name: "id", value: "this-attr" }],
///     data_location: text,
///     regex: { re: ".*", replace_with: "hello, ${1}!"
/// }
/// ```
/// will match
/// ```text
/// <div>
///     <b id="this-attr">
///         world
///     </b>
/// </div>
/// ```
/// and return "hello, world!"
#[derive(Clone, Debug)]
pub struct ElementDataQuery {
	/// Whether the query is optional. Ignore the fact it could've not been found if it is
	pub optional: bool,
	/// The queries to match against, one by one
	pub query: Vec<ElementQuery>,
	/// location of the data to extract
	pub data_location: DataLocation,
	/// optional [`Replace`] transform
	pub regex: Option<Replace>,
}

/// Extention trait for `&[ElementQuery]` that adds a method that return a pretty Display implementation for itself
pub trait ElementQuerySliceExt {
	/// Return a type that implements Display for itself
	fn display(&self) -> ElementQuerySliceDisplay<'_>;
}

/// Display implementation for `&[ElementQuery]`
pub struct ElementQuerySliceDisplay<'a> {
	slice: &'a [ElementQuery],
}

impl ElementQuerySliceExt for [ElementQuery] {
	fn display(&self) -> ElementQuerySliceDisplay<'_> {
		ElementQuerySliceDisplay { slice: self }
	}
}

impl Display for ElementQuerySliceDisplay<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.slice.is_empty() {
			return write!(f, "[]");
		}

		writeln!(f, "[")?;
		for (i, elem) in self.slice.iter().enumerate() {
			write!(f, "    #{}: ", i + 1)?;

			match &elem.kind {
				ElementKind::Tag(t) => write!(f, "<{t}/>")?,
				ElementKind::Class(c) => write!(f, "<tag class=\"{c}\">")?,
				ElementKind::Attr { name, value } => write!(f, "<tag {name}=\"{value}\"/>")?,
			}

			writeln!(f, ",")?;
		}
		writeln!(f, "]")?;

		Ok(())
	}
}
