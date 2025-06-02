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

/// A [`fetcher`](`crate`) primitive that contains a message and an id returned from a source that can be send to a sink.
///
/// Supports building with builders as well as a shorthand to avoid calling `.build()` on the message.
/// For example, this works:
/// ```
/// # use fetcher::entry::Entry;
/// # use fetcher::sinks::Message;
/// let _entry = Entry::builder()
///     .id("id".to_owned())
///     .msg(Message::builder()
///         .body("message body".to_owned()))  // notice no `.build()` on the message builder
///     .build(); // this `.build()` builds both
/// ```
#[derive(PartialEq, Eq, Clone, Default, bon::Builder)]
pub struct Entry {
	/// ID of the entry
	///
	/// A [`ReadFilter`](`crate::read_filter::ReadFilter`) can use it to differentiate already read entries from the unread ones.
	/// It is also used to map between entries and messages to support, e.g. replies
	#[builder(required, default, setters(
		name = id_internal,
		vis = "",
	))]
	pub id: Option<EntryId>,

	/// An entry this entry is replying to/quoting
	pub reply_to: Option<EntryId>,

	/// Raw contents gotten from a [`Source`](`crate::sources::Source`)
	///
	/// It's used to compose a message using [`transformators`](`crate::actions::transforms::Transform`).
	pub raw_contents: Option<String>,

	/// The message itself
	#[builder(into, default)]
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

impl<S: entry_builder::State> EntryBuilder<S> {
	/// ID of the entry
	///
	/// A [`ReadFilter`](`crate::read_filter::ReadFilter`) can use it to differentiate already read entries from the unread ones.
	/// It is also used to map between entries and messages to support, e.g. replies
	pub fn id(self, entry_id: String) -> EntryBuilder<entry_builder::SetId<S>>
	where
		S::Id: entry_builder::IsUnset,
	{
		self.id_internal(EntryId::new(entry_id))
	}

	/// ID of the entry
	///
	/// A [`ReadFilter`](`crate::read_filter::ReadFilter`) can use it to differentiate already read entries from the unread ones.
	/// It is also used to map between entries and messages to support, e.g. replies
	pub fn id_raw(self, entry_id: EntryId) -> EntryBuilder<entry_builder::SetId<S>>
	where
		S::Id: entry_builder::IsUnset,
	{
		self.id_internal(Some(entry_id))
	}
}
