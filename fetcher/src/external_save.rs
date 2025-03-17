/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ExternalSave`] trait that implementors can use to add a way to save read filter data and entry to message map externally,

use std::{
	collections::HashMap,
	convert::Infallible,
	fmt::{Debug, Display},
	io,
};

use crate::{entry::EntryId, sink::message::MessageId, utils::DisplayDebug};

/// This trait represent some kind of external save destination.
/// A way to preserve the state of a read filter, i.e. what has and has not been read, across restarts.
pub trait ExternalSave: Debug + Send + Sync {
	/// This function will be called every time something has been marked as read and should be saved externally
	///
	/// # Errors
	/// It may return an error if there has been issues saving, e.g. writing to disk
	async fn save_read_filter<RF>(&mut self, read_filter: &RF) -> Result<(), ExternalSaveError>;

	/// Save the entry id to message id map (see [`Task.entry_to_msg_map`]) enternally
	async fn save_entry_to_msg_map(
		&mut self,
		map: &HashMap<EntryId, MessageId>,
	) -> Result<(), ExternalSaveError>;
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("Can't save externally{}{}", .path.is_some().then_some(": ").unwrap_or_default(), if let Some(path) = .path.as_ref() { path as &dyn Display } else { &"" })]
pub struct ExternalSaveError {
	/// Inner IO error
	pub source: io::Error,

	/// Path/URL/some other kind of identifier of the location of the error
	pub path: Option<Box<dyn DisplayDebug + Send + Sync>>,
}

impl ExternalSave for Infallible {
	async fn save_read_filter<RF>(&mut self, _read_filter: &RF) -> Result<(), ExternalSaveError> {
		unreachable!()
	}

	/// Save the entry id to message id map (see [`Task.entry_to_msg_map`]) enternally
	async fn save_entry_to_msg_map(
		&mut self,
		_map: &HashMap<EntryId, MessageId>,
	) -> Result<(), ExternalSaveError> {
		unreachable!()
	}
}
