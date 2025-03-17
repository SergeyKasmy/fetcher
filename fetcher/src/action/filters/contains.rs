/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Contains`] filter

use regex::Regex;
use std::borrow::Cow;

use super::Filter;
use crate::{action::transforms::field::Field, entry::Entry, error::BadRegexError};

/// Filter out all entries whose field doesn't match the regular expression
#[derive(Clone, Debug)]
pub struct Contains {
	/// The regular expression to match
	pub re: Regex,

	/// The field that the regex should be matched against
	pub field: Field,
}

impl Contains {
	/// Create a new [`Contains`] with regular expression `re` that should be matched against `field`
	///
	/// # Errors
	/// if the regex is invalid
	pub fn new(regex: &str, field: Field) -> Result<Self, BadRegexError> {
		Ok(Self {
			re: Regex::new(regex)?,
			field,
		})
	}
}

impl Filter for Contains {
	/// Filter out some entries out of the `entries` vector
	async fn filter(&self, entries: &mut Vec<Entry>) {
		entries.retain(|ent| {
			let field = match self.field {
				Field::Title => ent.msg.title.as_deref().map(Cow::Borrowed),
				Field::Body => ent.msg.body.as_deref().map(Cow::Borrowed),
				Field::Link => ent.msg.link.as_ref().map(|s| Cow::Owned(s.to_string())),
				Field::Id => ent.id.as_ref().map(|id| Cow::Borrowed(id.0.as_str())),
				Field::ReplyTo => ent.reply_to.as_ref().map(|id| Cow::Borrowed(id.0.as_str())),
				Field::RawContets => ent.raw_contents.as_deref().map(Cow::Borrowed),
			};

			match field {
				Some(field) => self.re.is_match(&field),
				None => false,
			}
		});
	}
}
