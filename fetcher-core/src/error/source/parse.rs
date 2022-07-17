/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("RSS parsing error")]
	Rss(#[from] rss::Error),

	#[error("HTML parsing error")]
	Html(#[from] HtmlError),

	#[error("JSON parsing error")]
	Json(#[from] JsonError),
}

#[derive(thiserror::Error, Debug)]
pub enum HtmlError {
	#[error("URL not found")]
	UrlNotFound,

	#[error("Invalid URL")]
	InvalidUrl(#[from] url::ParseError),

	#[error("ID not found")]
	IdNotFound,

	#[error("Image not found but it's not optional")]
	ImageNotFound,

	#[error("Invalid time format")]
	InvalidTimeFormat(#[from] chrono::ParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum JsonError {
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

	#[error("ID not found")]
	InvalidUrl(#[from] url::ParseError),
}
