/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic building blog of [`fetcher`](`crate`) - [`Entry`]
//! that is passed throughout the program and that all modules either create, modify, or consume

pub mod id;
pub use id::EntryId;

use crate::{safe_slice::SafeSliceUntilExt, sinks::message::Message};

use std::fmt::Debug;

/// A [`fetcher`](`crate`) primitive that contains a message and an id returned from a source that can be send to a sink
#[derive(PartialEq, Eq, Clone, Default)]
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

impl Debug for Entry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Entry")
			.field("id", &self.id)
			.field("reply_to", &self.reply_to)
			.field(
				"raw_contents",
				&self
					.raw_contents
					.as_ref()
					.map(|s| s.pretty_slice_until(250)),
			)
			.field("msg", &self.msg)
			.finish()
	}
}
