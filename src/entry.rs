/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic building blog of [`fetcher`](`crate`) - [`Entry`]
//! that is passed throughout the program and that all modules either create, modify, or consume

use crate::sinks::message::Message;

use non_non_full::NonEmptyString;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Deref};

/// A [`fetcher`](`crate`) primitive that contains a message and an id returned from a source that can be send to a sink
#[derive(Clone, Default)]
pub struct Entry {
	/// ID of the entry
	///
	/// A [`ReadFilter`](`crate::read_filter::ReadFilter`) can use it to differentiate already read entries from the unread ones.
	/// It is also used to map between entries and messages to support, e.g. replies
	pub id: Option<EntryId>,

	/// An entry this entry is replying to/quoting
	pub reply_to: Option<EntryId>,

	/// Raw contents gotten from a [`Source`](`crate::sources::Source`)
	///
	/// It's used to compose a message using [`transformators`](`crate::actions::transforms::Transform`).
	pub raw_contents: Option<String>,

	/// The message itself
	pub msg: Message,
}

// TODO: make generic over String/i64/other types of id
/// An ID that can identify and entry to differentiate it from another one
#[derive(PartialEq, Eq, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct EntryId(pub NonEmptyString);

impl Deref for EntryId {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0.as_str()
	}
}

impl From<NonEmptyString> for EntryId {
	fn from(value: NonEmptyString) -> Self {
		Self(value)
	}
}

impl TryFrom<&str> for EntryId {
	// TODO: better error
	type Error = ();

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		if value.is_empty() {
			Err(())
		} else {
			Ok(Self(
				NonEmptyString::new(value.to_owned()).expect("should not be empty"),
			))
		}
	}
}

impl TryFrom<String> for EntryId {
	// TODO: better error
	type Error = ();

	fn try_from(value: String) -> Result<Self, Self::Error> {
		if value.is_empty() {
			Err(())
		} else {
			Ok(Self(
				NonEmptyString::new(value).expect("should not be empty"),
			))
		}
	}
}

impl Debug for Entry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Entry")
			.field("id", &self.id)
			.field("reply_to", &self.reply_to)
			.field("raw_contents.is_some()", &self.raw_contents.is_some())
			.field("msg", &self.msg)
			.finish()
	}
}
