/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(missing_docs)]

use crate::{
	action::transform::{
		entry::{
			html::query::{
				DataLocation as HtmlDataLocation, ElementQuery as HtmlElemQuery,
				ElementQuerySliceExt,
			},
			json::Keys as JsonKeys,
		},
		field::Field,
	},
	entry::Entry,
	error::InvalidUrlError,
};

use std::convert::Infallible;

#[derive(thiserror::Error, Debug)]
#[error("Error transforming entry")]
pub struct Error {
	#[source]
	pub kind: Kind,
	pub original_entry: Entry,
}

#[derive(thiserror::Error, Debug)]
pub enum Kind {
	#[error("Message link is not a valid URL after transforming")]
	FieldLinkTransformInvalidUrl(#[source] InvalidUrlError),

	#[error("HTTP error")]
	Http(#[from] HttpError),

	#[error("Feed parsing error")]
	Feed(#[from] FeedError),

	#[error("HTML parsing error")]
	Html(#[from] HtmlError),

	#[error("JSON parsing error")]
	Json(#[from] JsonError),

	#[error("Regex error")]
	Regex(#[from] RegexError),
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
	// TODO: impl Display for Field
	#[error("Missing URL in the entry {0:?} field")]
	MissingUrl(Field),

	#[error("Invalid URL in the entry {0:?} field")]
	InvalidUrl(Field, #[source] InvalidUrlError),

	#[error(transparent)]
	Other(#[from] crate::source::error::HttpError),
}

#[derive(thiserror::Error, Debug)]
pub enum FeedError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error(transparent)]
	Other(#[from] feed_rs::parser::ParseFeedError),
}

#[derive(thiserror::Error, Debug)]
pub enum HtmlError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error("HTML element #{} not found. From query list: \n{}",
			.num + 1,
			.elem_list.display()
			)]
	ElementNotFound {
		num: usize,
		elem_list: Vec<HtmlElemQuery>,
	},

	#[error("Data not found at {data:?} in element fount at {}",
			.element.display())]
	DataNotFoundInElement {
		data: HtmlDataLocation,
		element: Vec<HtmlElemQuery>,
	},

	#[error("HTML element {0:?} is empty")]
	ElementEmpty(Vec<HtmlElemQuery>),

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),

	#[error(transparent)]
	RegexError(#[from] RegexError),
}

#[derive(thiserror::Error, Debug)]
pub enum JsonError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error("Invalid JSON")]
	Invalid(#[from] serde_json::error::Error),

	#[error("JSON key #{num} not found. From query list: {key_list:?}")]
	KeyNotFound { num: usize, key_list: JsonKeys },

	#[error("JSON key {key:?} wrong type: expected {expected_type}, found {found_type}")]
	KeyWrongType {
		key: JsonKeys,
		expected_type: &'static str,
		found_type: String,
	},

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),
}

#[derive(thiserror::Error, Debug)]
pub enum RegexError {
	#[error("Invalid regex pattern")]
	InvalidPattern(#[from] regex::Error),

	#[error("Missing regex capture group named <s>, e.g. (?P<s>.*)")]
	CaptureGroupMissing,

	#[error("No match found in {0:?}")]
	NoMatchFound(String),
}

#[derive(thiserror::Error, Debug)]
#[error("There's nothing to transform from")]
pub struct RawContentsNotSetError;

impl From<Infallible> for Kind {
	fn from(inf: Infallible) -> Self {
		match inf {}
	}
}
