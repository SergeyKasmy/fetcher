/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Local file source
//!
//! This module contains [`File`] source

use crate::entry::Entry;
use crate::error::source::Error as SourceError;

use std::path::PathBuf;

/// File source. Reads contents of a file and puts them into [`raw_contents`](`crate::entry::Entry::raw_contents`)
#[derive(Debug)]
pub struct File {
	/// Path of the file
	pub path: PathBuf,
}

impl File {
	/// Read data from a file from the file system, returning its contents in the [`Entry.raw_contents`] field
	#[tracing::instrument(skip_all)]
	pub async fn get(&self) -> Result<Entry, SourceError> {
		let text = tokio::fs::read_to_string(&self.path)
			.await
			.map(|s| s.trim().to_owned())
			.map_err(|e| SourceError::FileRead(e, self.path.clone()))?;

		Ok(Entry {
			raw_contents: Some(text),
			..Default::default()
		})
	}
}
