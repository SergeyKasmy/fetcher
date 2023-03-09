/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Source`]s that can fetch data and create new [`Entries`](`Entry`) out of it
// TODO: add google calendar source. Google OAuth2 is already implemented :)
// TODO: make a new fetch module

pub mod email;
pub mod file;
pub mod http;
pub mod reddit;
pub mod twitter;

pub mod error;

pub use self::{email::Email, file::File, http::Http, reddit::Reddit, twitter::Twitter};
pub use crate::exec::Exec;

use self::error::SourceError;
use crate::{
	entry::{Entry, EntryId},
	error::Error,
	read_filter::{MarkAsRead, ReadFilter},
};

use async_trait::async_trait;
use std::fmt::Debug;

/// A trait that defines a way to fetch entries as well as mark them as read afterwards
pub trait Source: Fetch + MarkAsRead + Debug + Send + Sync {}

/// A trait that defines a way to fetch (entries)[`Entry`]
#[async_trait]
pub trait Fetch: Debug + Send + Sync {
	// TODO: maybe, instead of returining a vec, add a &mut Vec output parameter
	// and maybe also a trait method get_vec() that automatically creates a new vec, fetches into it, and returns it
	/// Fetch all available entries from the source
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError>;
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
	pub rf: Option<RF>,
}

#[async_trait]
impl<F, RF> Fetch for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: ReadFilter,
{
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		self.source.fetch().await
	}
}

#[async_trait]
impl<F, RF> MarkAsRead for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: ReadFilter,
{
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), Error> {
		if let Some(rf) = &mut self.rf {
			rf.mark_as_read(id).await?;
		}

		Ok(())
	}

	async fn set_read_only(&mut self) {
		if let Some(rf) = &mut self.rf {
			rf.set_read_only().await;
		}
	}
}

impl<F, RF> Source for SourceWithSharedRF<F, RF>
where
	F: Fetch,
	RF: ReadFilter,
{
}

#[async_trait]
impl Fetch for String {
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		Ok(vec![Entry {
			raw_contents: Some(self.clone()),
			..Default::default()
		}])
	}
}

#[async_trait]
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
