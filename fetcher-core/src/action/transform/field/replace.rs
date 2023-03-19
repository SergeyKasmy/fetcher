/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Contains the [`Replace`] transform

use regex::Regex;
use std::{borrow::Cow, convert::Infallible};

use super::TransformField;
use crate::{action::transform::result::TransformResult, error::BadRegexError};

/// Replace this with "" when you want to remove all HTML tags
pub const HTML_TAG_RE: &str = "<[^>]*>";

/// Replace the first regular expression match with a string
#[derive(Debug)]
pub struct Replace {
	/// The regular expression to match
	re: Regex,

	/// The string to replace the matched part with
	with: String,
}

impl Replace {
	/// Create a new [`Replace`] with regular expression `re` that replaces matches with string `with`
	///
	/// # Errors
	/// if the regular expression `re` is invalid
	pub fn new(re: &str, with: String) -> Result<Self, BadRegexError> {
		Ok(Self {
			re: Regex::new(re)?,
			with,
		})
	}
}

impl TransformField for Replace {
	type Err = Infallible;

	fn transform_field(&self, old_val: Option<&str>) -> Result<TransformResult<String>, Self::Err> {
		Ok(TransformResult::New(old_val.map(|old| {
			self.re.replace_all(old, &self.with).into_owned()
		})))
	}
}

impl Replace {
	/// Replace `text` with the [`Replace::with`] if [`Replace::re`] matches
	#[must_use]
	pub fn replace<'a>(&self, text: &'a str) -> Cow<'a, str> {
		self.re.replace_all(text, &self.with)
	}
}
