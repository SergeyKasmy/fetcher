/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module provides [`AlwaysErrors`] source that always returns an error. Used for debug purposes

use super::{Fetch, Source, error::SourceError};
use crate::{
	entry::{Entry, EntryId},
	error::FetcherError,
	read_filter::MarkAsRead,
};

/// This is a debug source that always returns an error
#[derive(Debug)]
pub struct AlwaysErrors;

impl Fetch for AlwaysErrors {
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		Err(SourceError::Debug)
	}
}

impl MarkAsRead for AlwaysErrors {
	async fn mark_as_read(&mut self, _id: &EntryId) -> Result<(), FetcherError> {
		Ok(())
	}

	async fn set_read_only(&mut self) {}
}

impl Source for AlwaysErrors {}
