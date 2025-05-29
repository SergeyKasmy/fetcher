/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ExternalSave`] trait that implementors can use to add a way to save read filter data and entry to message map externally,

use std::{collections::HashMap, convert::Infallible, fmt::Debug, io};

use serde::Serialize;

use crate::{
	entry::EntryId,
	maybe_send::{MaybeSend, MaybeSendSync, MaybeSync},
	read_filter::ReadFilter,
	sinks::message::MessageId,
};

/// This trait represent some kind of external save destination.
/// A way to preserve the state of a read filter, i.e. what has and has not been read, across restarts.
pub trait ExternalSave: Debug + MaybeSendSync {
	/// This function will be called every time something has been marked as read and should be saved externally
	///
	/// # Errors
	/// It may return an error if there has been issues saving, e.g. writing to disk
	fn save_read_filter<RF>(
		&mut self,
		read_filter: &RF,
	) -> impl Future<Output = Result<(), ExternalSaveError>> + MaybeSend
	where
		RF: ReadFilter + Serialize;

	/// Save the entry id to message id map (see [`Task.entry_to_msg_map`]) enternally
	fn save_entry_to_msg_map(
		&mut self,
		map: &HashMap<EntryId, MessageId>,
	) -> impl Future<Output = Result<(), ExternalSaveError>> + MaybeSend;
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("Failed to save read filter state externally{}{}", .path.is_some().then_some(": ").unwrap_or_default(), .path.as_deref().unwrap_or_default())]
pub struct ExternalSaveError {
	/// Inner IO error
	pub source: io::Error,

	/// Path/URL/some other kind of identifier of the location of the error
	pub path: Option<String>,
}

impl ExternalSave for () {
	async fn save_read_filter<RF: MaybeSync>(
		&mut self,
		_read_filter: &RF,
	) -> Result<(), ExternalSaveError> {
		Ok(())
	}

	/// Save the entry id to message id map (see [`Task.entry_to_msg_map`]) enternally
	async fn save_entry_to_msg_map(
		&mut self,
		_map: &HashMap<EntryId, MessageId>,
	) -> Result<(), ExternalSaveError> {
		Ok(())
	}
}

impl ExternalSave for Infallible {
	async fn save_read_filter<RF>(&mut self, _read_filter: &RF) -> Result<(), ExternalSaveError>
	where
		RF: ReadFilter + Serialize,
	{
		match *self {}
	}

	async fn save_entry_to_msg_map(
		&mut self,
		_map: &HashMap<EntryId, MessageId>,
	) -> Result<(), ExternalSaveError> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl ExternalSave for ! {
	async fn save_read_filter<RF>(&mut self, _read_filter: &RF) -> Result<(), ExternalSaveError>
	where
		RF: ReadFilter + Serialize,
	{
		match *self {}
	}

	async fn save_entry_to_msg_map(
		&mut self,
		_map: &HashMap<EntryId, MessageId>,
	) -> Result<(), ExternalSaveError> {
		match *self {}
	}
}

impl<E> ExternalSave for Option<E>
where
	E: ExternalSave,
{
	async fn save_read_filter<RF>(&mut self, read_filter: &RF) -> Result<(), ExternalSaveError>
	where
		RF: ReadFilter + Serialize,
	{
		let Some(inner) = self else {
			return Ok(());
		};

		inner.save_read_filter(read_filter).await
	}

	async fn save_entry_to_msg_map(
		&mut self,
		map: &HashMap<EntryId, MessageId>,
	) -> Result<(), ExternalSaveError> {
		let Some(inner) = self else {
			return Ok(());
		};

		inner.save_entry_to_msg_map(map).await
	}
}

impl<E> ExternalSave for &mut E
where
	E: ExternalSave,
{
	fn save_read_filter<RF>(
		&mut self,
		read_filter: &RF,
	) -> impl Future<Output = Result<(), ExternalSaveError>> + MaybeSend
	where
		RF: ReadFilter + Serialize,
	{
		(*self).save_read_filter(read_filter)
	}

	fn save_entry_to_msg_map(
		&mut self,
		map: &HashMap<EntryId, MessageId>,
	) -> impl Future<Output = Result<(), ExternalSaveError>> + MaybeSend {
		(*self).save_entry_to_msg_map(map)
	}
}
