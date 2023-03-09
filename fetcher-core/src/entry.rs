/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic building blog of [`fetcher`](`crate`) - [`Entry`]
//! that is passed throughout the program and that all modules either create, modify, or consume

use crate::sink::message::Message;

use std::{fmt::Debug, ops::Deref};

// TODO: make generic over String/i64/other types of id
/// An ID that can identify and entry to differentiate it from another one
#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct EntryId(pub String);

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

	/// Raw contents gotten from a [`Source`](`crate::source::Source`)
	///
	/// It's used to compose a message using [`transformators`](`crate::action::transform::Transform`).
	pub raw_contents: Option<String>,

	/// The message itself
	pub msg: Message,
}

impl Deref for EntryId {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<String> for EntryId {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl From<&str> for EntryId {
	fn from(value: &str) -> Self {
		Self(value.to_owned())
	}
}

impl Debug for Entry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Entry")
			.field("id", &self.id)
			.field("raw_contents.is_some()", &self.raw_contents.is_some())
			.field("msg", &self.msg)
			.finish()
	}
}
