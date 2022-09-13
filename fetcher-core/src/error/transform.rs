/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{entry::Entry, source::with_shared_rf::http::TransformFromField};

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

	#[error("RSS parsing error")]
	Rss(#[from] RssError),

	#[error("HTML parsing error")]
	Html(#[from] HtmlError),

	#[error("JSON parsing error")]
	Json(#[from] JsonError),
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
pub enum RssError {
	#[error(transparent)]
	NothingToTransform(#[from] NothingToTransformError),

	#[error(transparent)]
	Rss(#[from] rss::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum HtmlError {
	#[error(transparent)]
	NothingToTransform(#[from] NothingToTransformError),

	#[error("URL not found")]
	UrlNotFound,

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),

	#[error("ID not found")]
	IdNotFound,

	#[error("Image not found but it's not optional")]
	ImageNotFound,

	#[error("Invalid regex pattern")]
	InvalidRegexPattern(#[from] regex::Error),

	#[error("Missing regex capture group named <s>")]
	RegexCaptureGroupMissing,

	#[error("Invalid time format")]
	InvalidTimeFormat(#[from] chrono::ParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum JsonError {
	#[error(transparent)]
	NothingToTransform(#[from] NothingToTransformError),

	#[error("Invalid JSON")]
	JsonParseInvalid(#[from] serde_json::error::Error),

	#[error("JSON key {0} not found")]
	JsonParseKeyNotFound(String),

	#[error("JSON key {key} wrong type: expected {expected_type}, found {found_type}")]
	JsonParseKeyWrongType {
		key: String,
		expected_type: &'static str,
		found_type: String,
	},

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),
}

#[derive(thiserror::Error, Debug)]
#[error("There's nothing to transform")]
pub struct NothingToTransformError;

#[derive(thiserror::Error, Debug)]
#[error("Invalid URL: {1}")]
pub struct InvalidUrlError(#[source] pub url::ParseError, pub String);
