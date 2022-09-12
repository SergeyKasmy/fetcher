/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::TransformField;

use crate::{action::transform::result::TransformResult, error::transform::RegexError};

#[derive(Debug)]
pub struct Regex {
	pub re: regex::Regex,
	pub action: Action,
}

#[derive(Debug)]
pub enum Action {
	Find,
	Extract { passthrough_if_not_found: bool },
}

impl Regex {
	pub fn new(re: &str, action: Action) -> Result<Self, RegexError> {
		tracing::trace!("Creating Regex transform with str {:?}", re);
		Ok(Self {
			re: regex::Regex::new(re)?,
			action,
		})
	}
}

impl TransformField for Regex {
	type Error = RegexError;

	fn transform_field(&self, field: &str) -> Result<TransformResult<String>, RegexError> {
		Ok(TransformResult::New(
			self.run(field)?.map(ToOwned::to_owned),
		))
	}
}

impl Regex {
	pub fn run<'a>(&self, text: &'a str) -> Result<Option<&'a str>, RegexError> {
		let res = match (&self.action, extract(&self.re, text)) {
			// return the original str if a match was found or even extracted from some reason when we are just searching
			(Action::Find, ExtractionResult::Matched | ExtractionResult::Extracted(_)) => {
				Some(text)
			}
			// return the extracted str if we are just extracting
			(Action::Extract { .. }, ExtractionResult::Extracted(extracted_s)) => Some(extracted_s),
			// return the original str if we are extracting but passthrough_if_not_found is on
			(
				Action::Extract {
					passthrough_if_not_found,
				},
				ExtractionResult::Matched,
			) if *passthrough_if_not_found => Some(text),
			// return an error if we are extracting without passthrough and we haven't extracted anything
			(Action::Extract { .. }, ExtractionResult::Matched | ExtractionResult::NotMatched) => {
				return Err(RegexError::CaptureGroupMissing)
			}
			// return nothing if we haven't found anything
			(_, ExtractionResult::NotMatched) => None,
		};

		Ok(res)
	}
}

#[derive(Debug)]
pub(crate) enum ExtractionResult<'a> {
	NotMatched,
	Matched,
	Extracted(&'a str),
}

pub(crate) fn extract<'a>(re: &regex::Regex, text: &'a str) -> ExtractionResult<'a> {
	match re.captures(text) {
		Some(capture_groups) => match capture_groups.name("s") {
			Some(s) => ExtractionResult::Extracted(s.as_str()),
			None => ExtractionResult::Matched,
		},
		None => ExtractionResult::NotMatched,
	}
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
	use super::*;

	use assert_matches::assert_matches;

	#[test]
	fn extract_single() {
		let re = Regex::new(
			"Hello, (?P<s>.*)!",
			Action::Extract {
				passthrough_if_not_found: false,
			},
		)
		.unwrap();
		let s = "Hello, world!";

		assert_matches!(extract(&re.re, s), ExtractionResult::Extracted("world"));
	}

	#[test]
	fn extract_not_found() {
		let re = Regex::new(
			"Hello, (?P<s>.*)!",
			Action::Extract {
				passthrough_if_not_found: false,
			},
		)
		.unwrap();
		let s = "Bad string";

		assert_matches!(extract(&re.re, s), ExtractionResult::NotMatched);
	}
}
