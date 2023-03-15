/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use regex::Regex;

use super::TransformField;
use crate::{action::transform::result::TransformResult, error::BadRegexError};

const CAPTURE_GROUP_NAME: &str = "e";

/// Extract the contents of capture group <[`CAPTURE_GROUP_NAME`]> using a regular expression
#[derive(Debug)]
pub struct Extract {
	/// The regular expression to match against. Replace the value with the value matched inside [`CAPTURE_GROUP_NAME`] capture group
	re: Regex,

	/// Passthrough the old value if the regex didn't match
	passthrough_if_not_found: bool,
}

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum ExtractError {
	#[error(transparent)]
	BadRegex(#[from] BadRegexError),

	#[error("Missing capture group <{CAPTURE_GROUP_NAME}>, e.g. (?P<e>.*) in regular expression: {re:?}")]
	CaptureGroupMissing { re: String },

	#[error("Capture group not found but passthrough_if_not_found is not set")]
	CaptureGroupNotFound,
}

impl Extract {
	/// Create a new [`Extract`] with regular expression `re` and `passthrough_if_not_found`
	///
	/// # Errors
	/// * if the regex is invalid
	/// * if the regex doesn't contains capture group <[`CAPTURE_GROUP_NAME`]>
	pub fn new(re: &str, passthrough_if_not_found: bool) -> Result<Self, ExtractError> {
		let re_raw = re;
		let re = Regex::new(re_raw).map_err(BadRegexError)?;

		if !re
			.capture_names()
			.any(|capt| capt.map_or(false, |capt| capt == CAPTURE_GROUP_NAME))
		{
			return Err(ExtractError::CaptureGroupMissing {
				re: re_raw.to_owned(),
			});
		}

		Ok(Self {
			re,
			passthrough_if_not_found,
		})
	}
}

impl TransformField for Extract {
	type Err = ExtractError;

	fn transform_field(&self, old_val: Option<&str>) -> Result<TransformResult<String>, Self::Err> {
		let Some(field) = old_val else {
			#[allow(clippy::redundant_else)]
			if self.passthrough_if_not_found {
				// retain the old value
				return Ok(TransformResult::Old(None));
			} else {
				// override with none
				return Ok(TransformResult::New(None));
			}
		};

		let extracted = match self.re.captures(field) {
			Some(captures) => match captures.name(CAPTURE_GROUP_NAME) {
				Some(matched) => matched.as_str(),
				None if self.passthrough_if_not_found => field,
				None => todo!("error"),
			},
			None => todo!("error"),
		};

		Ok(TransformResult::New(Some(extracted.to_owned())))
	}
}
