/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{entry::Entry, error::transform::RegexError, sink::Message};

#[derive(Debug)]
pub struct Regex {
	pub re: regex::Regex,
	pub passthrough_if_not_found: bool,
}

impl Regex {
	pub fn new(re: &str, passthrough_if_not_found: bool) -> Result<Self, RegexError> {
		tracing::trace!("Creating Regex transform with str {:?}", re);
		Ok(Self {
			re: regex::Regex::new(re)?,
			passthrough_if_not_found,
		})
	}

	pub fn transform(&self, entry: &Entry) -> Result<Entry, RegexError> {
		tracing::trace!("Transforming entry {:#?}", entry);

		let body = match entry.msg.body.clone() {
			Some(b) => self
				.extract(&b)?
				.filter(|s| !s.is_empty())
				.map(ToOwned::to_owned),
			None => None,
		};

		tracing::trace!("Body after: {:?}", body);

		Ok(Entry {
			msg: Message {
				body,
				..Default::default()
			},
			..Default::default()
		})
	}

	pub(crate) fn extract<'a>(&self, text: &'a str) -> Result<Option<&'a str>, RegexError> {
		let text = match self.re.captures(text) {
			Some(capture_groups) => Some(
				capture_groups
					.name("s")
					.ok_or(RegexError::CaptureGroupMissing)?
					.as_str(),
			),
			None if self.passthrough_if_not_found => Some(text),
			None => None,
		};

		Ok(text)
	}
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn extract() {
		let re = Regex::new("Hello, (?P<s>.*)!", false).unwrap();
		let s = "Hello, world!";

		let res = re
			.extract(s)
			.expect("An error has happened during string capture")
			.expect("Nothing has been found");

		assert_eq!(res, "world");
	}

	#[test]
	fn not_found() {
		let re = Regex::new("Hello, (?P<s>.*)!", false).unwrap();
		let s = "Bad string";

		let res = re
			.extract(s)
			.expect("An error has happened during string capture");

		assert!(matches!(res, None));
	}

	#[test]
	fn not_found_remained() {
		let re = Regex::new("Hello, (?P<s>.*)!", true).unwrap();
		let s = "Bad string";

		let res = re
			.extract(s)
			.expect("An error has happened during string capture")
			.unwrap();

		assert_eq!(res, "Bad string");
	}
}
