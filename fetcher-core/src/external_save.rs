/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ExternalSave`] trait that implementors can use to add a way to save read filter data and entry to message map externally,

use async_trait::async_trait;
use std::{collections::HashMap, fmt::Debug, io};

use crate::{entry::EntryId, read_filter::ReadFilter, sink::message::MessageId};

/// This trait represent some kind of external save destination.
/// A way to preserve the state of a read filter, i.e. what has and has not been read, across restarts.
#[async_trait]
pub trait ExternalSave: Debug + Send + Sync {
	/// This function will be called every time something has been marked as read and should be saved externally
	///
	/// # Errors
	/// It may return an error if there has been issues saving, e.g. writing to disk
	async fn save_read_filter(&mut self, read_filter: &dyn ReadFilter) -> io::Result<()>;

	/// Save the entry id to message id map (see [`Task.entry_to_msg_map`]) enternally
	async fn save_entry_to_msg_map(&mut self, map: &HashMap<EntryId, MessageId>) -> io::Result<()>;
}
