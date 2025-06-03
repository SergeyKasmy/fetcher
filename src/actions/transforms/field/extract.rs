/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Extract`] field transform, as well as all errors that can happen while creating or executing it

use regex::Regex;

use super::TransformField;
use crate::{actions::transforms::result::TransformResult, error::BadRegexError};

/// Extract the contents of capture groups using a regular expression and concat them
#[derive(Debug)]
pub struct Extract {
	/// The regular expression to match against. Replace the value of the field with the contents of capture groups
	re: Regex,

	/// Passthrough the old value if the regex didn't match
	passthrough: PassthroughIfNotMatched,
}

/// What happens if the regex doesn't match?
#[derive(Debug)]
pub enum PassthroughIfNotMatched {
	/// Always passthrough the old field value, even if the regex didn't match
	Always,

	/// Return an error if the regex didn't match
	ReturnError,
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum ExtractError {
	#[error(transparent)]
	BadRegex(#[from] BadRegexError),

	// Rename to "Regex not matched"
	#[error("Capture group not found but passthrough_if_not_found is not set")]
	CaptureGroupNotFound,
}

impl Extract {
	/// Create a new [`Extract`] with regular expression `re` and `passthrough_if_not_found`
	///
	/// # Errors
	/// * if the regex is invalid
	pub fn new(re: &str, passthrough: PassthroughIfNotMatched) -> Result<Self, ExtractError> {
		let re = Regex::new(re).map_err(BadRegexError)?;

		Ok(Self { re, passthrough })
	}
}

impl TransformField for Extract {
	type Err = ExtractError;

	async fn transform_field(
		&mut self,
		value: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		let Some(field) = value else {
			return Ok(TransformResult::Previous);
		};

		let extracted = match extract_captures_from(&self.re, field) {
			Some(v) => v,
			None if matches!(self.passthrough, PassthroughIfNotMatched::Always) => field.to_owned(),
			None => return Err(ExtractError::CaptureGroupNotFound),
		};

		Ok(TransformResult::New(extracted))
	}
}

/// Extracts and concats all capture groups into a single string
fn extract_captures_from(regex: &Regex, from: &str) -> Option<String> {
	regex.captures(from).map(|captures| {
		captures
			.iter()
			.skip(1 /* the first match that matches the entire regex */)
			.filter_map(|capt| Some(capt?.as_str()))
			.collect()
	})
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;

	const FROM: &str = "HelloxWorld";

	#[test]
	fn one() {
		let re = Regex::new("(?s)(.*)x").unwrap();
		assert_eq!(extract_captures_from(&re, FROM).unwrap(), "Hello");
	}

	#[test]
	fn several() {
		let re = Regex::new("(?s)(.*)x(.*)").unwrap();
		assert_eq!(extract_captures_from(&re, FROM).unwrap(), "HelloWorld");
	}

	#[test]
	fn not_matched() {
		let re = Regex::new("(?s)(.*)xxx(.*)").unwrap();
		assert!(extract_captures_from(&re, FROM).is_none());
	}
}
