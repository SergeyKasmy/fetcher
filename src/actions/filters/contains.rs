/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Contains`] filter

use regex::Regex;
use std::{borrow::Cow, convert::Infallible};

use super::{Filter, FilterableEntries};
use crate::{actions::transforms::field::Field, error::BadRegexError};

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
	type Err = Infallible;

	/// Filter out some entries out of the `entries` vector
	async fn filter(&mut self, mut entries: FilterableEntries<'_>) -> Result<(), Self::Err> {
		entries.retain(|ent| {
			let field = match self.field {
				Field::Title => ent.msg.title.as_deref().map(Cow::Borrowed),
				Field::Body => ent.msg.body.as_deref().map(Cow::Borrowed),
				Field::Link => ent.msg.link.as_ref().map(|s| Cow::Owned(s.clone())),
				Field::Id => ent.id.as_ref().map(|id| Cow::Borrowed(id.0.as_str())),
				Field::ReplyTo => ent.reply_to.as_ref().map(|id| Cow::Borrowed(id.0.as_str())),
				Field::RawContents => ent.raw_contents.as_deref().map(Cow::Borrowed),
			};

			field.is_some_and(|field| self.re.is_match(&field))
		});

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		actions::{
			filters::{Filter, FilterableEntries},
			transforms::field::Field,
		},
		entry::Entry,
		sinks::message::Message,
	};

	use super::Contains;

	#[tokio::test]
	async fn contains() {
		let bodies = [
			"Hello, World!",
			"Hello, Earth!",
			"Bye, World!",
			"Bye, Earth!",
		];

		let mut entries = bodies
			.into_iter()
			.map(|body| {
				Entry::builder()
					.msg(Message::builder().body(body.to_owned()))
					.build()
			})
			.collect::<Vec<_>>();

		let mut contains = Contains::new("World", Field::Body).unwrap();

		contains
			.filter(FilterableEntries::new(&mut entries))
			.await
			.unwrap();

		assert_eq!(
			entries
				.iter()
				.map(|ent| ent.msg.body.as_ref().unwrap().as_str())
				.collect::<Vec<_>>(),
			["Hello, World!", "Bye, World!"]
		);
	}
}
