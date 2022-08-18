/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::sink::Message;

// TODO: add message history via responce id -> message id hashmap

/// A [`fetcher`](`crate`) primitive that contains a message and an id returned from a source that can be send to a sink
#[derive(Clone, Debug)]
pub struct Entry {
	/// An optional id of that entry. A [`ReadFilter`](`crate::read_filter::ReadFilter`) can use it to differentiate already read entries from the unread ones
	pub id: Option<String>, // TODO: add date id type
	/// The message itself
	pub msg: Message,
}
