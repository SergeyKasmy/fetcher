/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Source`]s that can fetch data and create new [`Entries`](`Entry`) out of it
// TODO: add google calendar source. Google OAuth2 is already implemented :)

pub mod always_errors;
pub mod email;
pub mod file;
pub mod http;
pub mod reddit;

pub mod error;

pub use self::{email::Email, file::File, http::Http, reddit::Reddit};
pub use crate::exec::Exec;

use self::error::SourceError;
use crate::{
	entry::{Entry, EntryId},
	error::FetcherError,
	maybe_send::{MaybeSend, MaybeSendSync},
	read_filter::{MarkAsRead, ReadFilter},
};

use std::fmt::Debug;

/// A trait that defines a way to fetch entries as well as mark them as read afterwards
pub trait Source: Fetch + MarkAsRead + Debug + MaybeSendSync {}

/// A trait that defines a way to fetch (entries)[`Entry`]
pub trait Fetch: Debug + MaybeSendSync {
	/// Fetch all available entries from the source
	fn fetch(&mut self) -> impl Future<Output = Result<Vec<Entry>, SourceError>> + MaybeSend;

	fn into_source_with_read_filter<RF>(self, read_filter: RF) -> SourceWithSharedRF<Self, RF>
	where
		Self: Sized,
		RF: ReadFilter,
	{
		SourceWithSharedRF {
			source: self,
			rf: read_filter,
		}
	}

	fn into_source_without_read_filter(self) -> SourceWithSharedRF<Self, ()>
	where
		Self: Sized,
	{
		SourceWithSharedRF {
			source: self,
			rf: (),
		}
	}
}

impl<F, RF> Fetch for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: ReadFilter,
{
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		self.source.fetch().await
	}
}

impl<F, RF> MarkAsRead for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: ReadFilter,
{
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError> {
		self.rf.mark_as_read(id).await
	}

	#[expect(
		clippy::semicolon_if_nothing_returned,
		reason = "just forwards the method call, should return the same value"
	)]
	async fn set_read_only(&mut self) {
		self.rf.set_read_only().await
	}
}

impl<F, RF> Source for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: ReadFilter,
{
}

impl Fetch for String {
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		Ok(vec![Entry {
			raw_contents: Some(self.clone()),
			..Default::default()
		}])
	}
}

impl<T> Fetch for Vec<T>
where
	T: Fetch,
{
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		let mut entries = Vec::new();

		for fetch in self {
			entries.extend(fetch.fetch().await?);
		}

		Ok(entries)
	}
}

/// A wrapper around a [`Fetch`] that uses an external way to filter read entries,
/// as well as a (read filter)[`ReadFilter`]
#[derive(Debug)]
pub struct SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: ReadFilter,
{
	/// The source to fetch data from
	pub source: F,

	/// The read filter that's used to mark entries as read
	pub rf: RF,
}
