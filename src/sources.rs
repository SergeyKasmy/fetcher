/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Source`]s that can fetch data and create new [`Entries`](`Entry`) out of it
// TODO: add google calendar source. Google OAuth2 is already implemented :)

pub mod error;

pub mod file;

pub use self::file::File;
pub use crate::exec::Exec;

#[cfg(feature = "source-email")]
pub mod email;
#[cfg(feature = "source-email")]
pub use self::email::Email;

#[cfg(feature = "source-reddit")]
pub mod reddit;
#[cfg(feature = "source-reddit")]
pub use self::reddit::Reddit;

#[cfg(feature = "source-http")]
pub mod http;
#[cfg(feature = "source-http")]
pub use self::http::Http;

use self::error::SourceError;
use crate::{
	entry::{Entry, EntryId},
	maybe_send::{MaybeSend, MaybeSendSync},
	read_filter::MarkAsRead,
};

use std::convert::Infallible;

/// A trait that defines a way to fetch entries as well as mark them as read afterwards
pub trait Source: Fetch + MarkAsRead + MaybeSendSync {}

/// A trait that defines a way to fetch (entries)[`Entry`]
pub trait Fetch: MaybeSendSync {
	/// Error that may be returned. Returns [`Infallible`](`std::convert::Infallible`) if it never errors
	type Err: Into<SourceError>;

	/// Fetches all available entries from the source
	fn fetch(&mut self) -> impl Future<Output = Result<Vec<Entry>, Self::Err>> + MaybeSend;

	/// Converts the value into a source with the provided [`ReadFilter`].
	fn into_source_with_read_filter<RF>(self, read_filter: RF) -> SourceWithSharedRF<Self, RF>
	where
		Self: Sized,
		RF: MarkAsRead,
	{
		SourceWithSharedRF {
			source: self,
			rf: Some(read_filter),
		}
	}

	/// Converts the value into a source without support for filtering read/unread entries.
	fn into_source_without_read_filter(self) -> SourceWithSharedRF<Self, ()>
	where
		Self: Sized,
	{
		SourceWithSharedRF {
			source: self,
			rf: None,
		}
	}
}

/// A wrapper around a [`Fetch`] that uses an external way to filter read entries,
/// as well as a (read filter)[`ReadFilter`]
#[derive(Debug)]
pub struct SourceWithSharedRF<F, RF> {
	/// The source to fetch data from
	pub source: F,

	/// The read filter that's used to mark entries as read
	pub rf: Option<RF>,
}

impl Fetch for String {
	type Err = Infallible;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		let entry = Entry::builder().raw_contents(self.clone()).build();
		Ok(vec![entry])
	}
}

impl<T> Fetch for Vec<T>
where
	T: Fetch,
{
	type Err = SourceError;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		let mut entries = Vec::new();

		for fetch in self {
			entries.extend(fetch.fetch().await.map_err(Into::into)?);
		}

		Ok(entries)
	}
}

impl Fetch for () {
	type Err = Infallible;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		Ok(vec![Entry::default()])
	}
}

impl Fetch for Infallible {
	type Err = Infallible;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl Fetch for ! {
	type Err = !;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		match *self {}
	}
}

impl<F> Fetch for Option<F>
where
	F: Fetch,
{
	type Err = F::Err;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		let Some(inner) = self else {
			return ().fetch().await.map_err(|e| match e {});
		};

		inner.fetch().await
	}
}
impl<F> Fetch for &mut F
where
	F: Fetch,
{
	type Err = F::Err;

	fn fetch(&mut self) -> impl Future<Output = Result<Vec<Entry>, Self::Err>> + MaybeSend {
		(*self).fetch()
	}
}

impl<F, RF> Source for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: MarkAsRead,
{
}

impl Source for () {}
impl Source for Infallible {}
#[cfg(feature = "nightly")]
impl Source for ! {}
impl<S> Source for Option<S> where S: Source {}
impl<S> Source for &mut S where S: Source {}

impl<F, RF> Fetch for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: MarkAsRead,
{
	type Err = F::Err;

	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		self.source.fetch().await
	}
}

impl<F, RF> MarkAsRead for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: MarkAsRead,
{
	type Err = RF::Err;

	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Self::Err> {
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
