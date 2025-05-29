/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`MarkAsRead`] trait

use std::{convert::Infallible, error::Error as StdError, fmt::Debug};

use crate::{
	entry::EntryId,
	external_save::ExternalSaveError,
	maybe_send::{MaybeSend, MaybeSendSync},
};

#[cfg(feature = "source-email")]
use crate::sources::email::ImapError;

/// A trait that defines a way to mark an entry as read
pub trait MarkAsRead: MaybeSendSync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<MarkAsReadError>;

	/// Mark the entry with `id` as read
	fn mark_as_read(
		&mut self,
		id: &EntryId,
	) -> impl Future<Output = Result<(), Self::Err>> + MaybeSend;

	/// Set the current "mark as read"er to read only mode
	fn set_read_only(&mut self) -> impl Future<Output = ()> + MaybeSend;
}

/// All errors that could happen while marking an entry as read
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum MarkAsReadError {
	#[cfg(feature = "source-email")]
	#[error("Failed to mark the email as read")]
	Imap(#[from] ImapError),

	#[error(transparent)]
	ExternalSave(#[from] ExternalSaveError),

	#[error(transparent)]
	Other(#[from] Box<dyn StdError + Send + Sync>),
}

impl MarkAsRead for () {
	type Err = Infallible;

	async fn mark_as_read(&mut self, _id: &EntryId) -> Result<(), Self::Err> {
		Ok(())
	}

	async fn set_read_only(&mut self) {}
}

impl<M: MarkAsRead> MarkAsRead for Option<M> {
	type Err = M::Err;

	#[tracing::instrument(skip(self))]
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Self::Err> {
		match self {
			Some(m) => m.mark_as_read(id).await?,
			None => {
				tracing::debug!("Ignoring mark as read request");
			}
		}

		Ok(())
	}

	async fn set_read_only(&mut self) {
		match self {
			Some(m) => m.set_read_only().await,
			None => {
				tracing::debug!("Ignoring set read only request");
			}
		}
	}
}

impl MarkAsRead for Infallible {
	type Err = Infallible;

	async fn mark_as_read(&mut self, _id: &EntryId) -> Result<(), Self::Err> {
		match *self {}
	}

	async fn set_read_only(&mut self) {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl MarkAsRead for ! {
	type Err = !;

	async fn mark_as_read(&mut self, _id: &EntryId) -> Result<(), Self::Err> {
		match *self {}
	}

	async fn set_read_only(&mut self) {
		match *self {}
	}
}

impl<M> MarkAsRead for &mut M
where
	M: MarkAsRead,
{
	type Err = M::Err;

	fn mark_as_read(
		&mut self,
		id: &EntryId,
	) -> impl Future<Output = Result<(), Self::Err>> + MaybeSend {
		(*self).mark_as_read(id)
	}

	fn set_read_only(&mut self) -> impl Future<Output = ()> + MaybeSend {
		(*self).set_read_only()
	}
}

impl From<Infallible> for MarkAsReadError {
	fn from(value: Infallible) -> Self {
		match value {}
	}
}

#[cfg(feature = "nightly")]
impl From<!> for MarkAsReadError {
	fn from(value: !) -> Self {
		match value {}
	}
}
