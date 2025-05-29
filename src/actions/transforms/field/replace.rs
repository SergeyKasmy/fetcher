/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Contains the [`Replace`] transform

use regex::Regex;
use std::{borrow::Cow, convert::Infallible};

use super::TransformField;
use crate::{
	StaticStr,
	actions::transforms::result::{OptionUnwrapTransformResultExt, TransformResult},
	error::BadRegexError,
};

/// Replace this with "" when you want to remove all HTML tags
pub const HTML_TAG_RE: &str = "<[^>]*>";

/// Replace the first regular expression match with a string
#[derive(Clone, Debug)]
pub struct Replace {
	/// The regular expression to match
	pub re: Regex,

	/// The string to replace the matched part with
	pub with: StaticStr,
}

impl Replace {
	/// Create a new [`Replace`] with regular expression `re` that replaces matches with string `with`
	///
	/// # Errors
	/// if the regular expression `re` is invalid
	pub fn new(re: &str, with: impl Into<StaticStr>) -> Result<Self, BadRegexError> {
		Ok(Self {
			re: Regex::new(re)?,
			with: with.into(),
		})
	}
}

impl TransformField for Replace {
	type Err = Infallible;

	fn transform_field(
		&mut self,
		old_val: Option<&str>,
	) -> Result<TransformResult<String>, Self::Err> {
		Ok(old_val
			.map(|old| self.re.replace_all(old, self.with.as_str()).into_owned())
			.unwrap_or_empty())
	}
}

impl Replace {
	/// Replace `text` with the [`Replace::with`] if [`Replace::re`] matches
	#[must_use]
	pub fn replace<'a>(&self, text: &'a str) -> Cow<'a, str> {
		self.re.replace_all(text, self.with.as_str())
	}
}
