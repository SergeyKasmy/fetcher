/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Local file source
//!
//! This module contains [`File`] source

use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

use super::{error::SourceError, Fetch};
use crate::entry::Entry;

/// File source. Reads contents of a file and puts them into [`raw_contents`](`crate::entry::Entry::raw_contents`)
#[derive(Debug)]
pub struct File {
	/// Path of the file
	pub path: PathBuf,
}

#[async_trait]
impl Fetch for File {
	/// Read data from a file from the file system, returning its contents in the [`Entry.raw_contents`] field
	// doesn't actually mutate itself
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		let text = fs::read_to_string(&self.path)
			.await
			.map(|s| s.trim().to_owned())
			.map_err(|e| SourceError::File(e, self.path.clone()))?;

		Ok(vec![Entry {
			raw_contents: Some(text),
			..Default::default()
		}])
	}
}
