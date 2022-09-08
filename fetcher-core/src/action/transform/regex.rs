/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Transform;
use crate::action::transform::result::{
	TransformResult as TrRes, TransformedEntry, TransformedMessage,
};
use crate::{entry::Entry, error::transform::RegexError};

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

impl Transform for Regex {
	type Error = RegexError;

	fn transform(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		// self.transform_impl(entry).map(|e| vec![e])
		let body = match entry.msg.body.clone() {
			Some(body) => body,
			None => {
				return Ok(vec![TransformedEntry {
					msg: TransformedMessage {
						body: TrRes::Empty,
						..Default::default()
					},
					..Default::default()
				}])
			}
		};

		let body = self.run(&body)?.map(ToOwned::to_owned);

		Ok(vec![TransformedEntry {
			msg: TransformedMessage {
				body: TrRes::New(body),
				..Default::default()
			},
			..Default::default()
		}])
	}
}

impl Regex {
	pub fn run<'a>(&self, text: &'a str) -> Result<Option<&'a str>, RegexError> {
		let res = match (&self.action, extract(&self.re, &text)) {
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
