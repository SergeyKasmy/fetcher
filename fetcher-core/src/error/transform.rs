/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(missing_docs)]

use crate::{
	action::transform::entry::{html::query::ElementQuery as HtmlElemQuery, json::Key as JsonKey},
	entry::Entry,
	source::http::TransformFromField,
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
	#[error("Missing URL in the entry {0} field")]
	MissingUrl(TransformFromField),

	#[error("Invalid URL in the entry raw_contents field")]
	InvalidUrl(#[from] InvalidUrlError),

	#[error(transparent)]
	Other(#[from] crate::error::source::HttpError),
}

#[derive(thiserror::Error, Debug)]
#[error("There's nothing to transform")]
pub struct NothingToTransformError;

#[derive(thiserror::Error, Debug)]
#[error("Invalid URL: {1}")]
pub struct InvalidUrlError(#[source] pub url::ParseError, pub String);

#[derive(thiserror::Error, Debug)]
pub enum FeedError {
	#[error(transparent)]
	NothingToTransform(#[from] NothingToTransformError),

	#[error(transparent)]
	Other(#[from] feed_rs::parser::ParseFeedError),
}

#[derive(thiserror::Error, Debug)]
pub enum HtmlError {
	#[error(transparent)]
	NothingToTransform(#[from] NothingToTransformError),

	#[error("HTML element {0:?} not found")]
	ElementNotFound(HtmlElemQuery),

	#[error("HTML element {0:?} is empty")]
	ElementEmpty(HtmlElemQuery),

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),

	#[error(transparent)]
	RegexError(#[from] RegexError),

	#[error("Invalid time format")]
	InvalidTimeFormat(#[from] chrono::ParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum JsonError {
	#[error(transparent)]
	NothingToTransform(#[from] NothingToTransformError),

	#[error("Invalid JSON")]
	Invalid(#[from] serde_json::error::Error),

	#[error("JSON key {0} not found")]
	KeyNotFound(JsonKey),

	#[error("JSON key {key} wrong type: expected {expected_type}, found {found_type}")]
	KeyWrongType {
		key: JsonKey,
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

impl From<Infallible> for Kind {
	fn from(inf: Infallible) -> Self {
		match inf {}
	}
}
