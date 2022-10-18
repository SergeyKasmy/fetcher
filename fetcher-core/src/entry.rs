/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the basic building blog of [`fetcher`](`crate`) - [`Entry`]
//! that is passed throughout the program and that all modules either create, modify, or consume

// TODO: add message history via responce id -> message id hashmap

use crate::sink::Message;

use std::fmt::Debug;

/// A [`fetcher`](`crate`) primitive that contains a message and an id returned from a source that can be send to a sink
#[derive(Clone, Default)]
pub struct Entry {
	/// An optional id of that entry. A [`ReadFilter`](`crate::read_filter::ReadFilter`) can use it to differentiate already read entries from the unread ones
	pub id: Option<String>, // TODO: add date id type

	/// Raw contents gotten from a [`Source`](`crate::source::Source`)
	///
	/// It's used to compose a message using [`transformators`](`crate::action::transform::Transform`).
	pub raw_contents: Option<String>,

	/// The message itself
	pub msg: Message,
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
