/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod file;
pub mod http;
pub mod twitter;

use self::{file::File, http::Http, twitter::Twitter};
use crate::{entry::Entry, error::source::Error as SourceError};

/// Always contains a vec with sources of the same type
#[derive(Debug)]
pub struct Source(Vec<Kind>);

#[derive(Debug)]
pub enum Kind {
	File(File),
	Http(Http),
	Twitter(Twitter),
}

impl Source {
	/// Create a new sources vec that contains one or several pure sources of the same type
	///
	/// # Errors
	/// * if the source list is empty
	/// * if the several sources that were provided are of different `WithStaredReadFilterInner` variants
	pub fn new(sources: Vec<Kind>) -> Result<Self, SourceError> {
		match sources.len() {
			0 => return Err(SourceError::EmptySourceList),
			1 => (),
			// assert that all source types are of the same enum variant
			_ => {
				for variants in sources.windows(2) {
					use std::mem::discriminant as disc;

					if disc(&variants[0]) != disc(&variants[1]) {
						return Err(SourceError::SourceListHasDifferentVariants);
					}
				}
			}
		}

		Ok(Self(sources))
	}

	/// Get all entries from the sources
	///
	/// # Errors
	/// if there was an error fetching from a source
	pub async fn get(&mut self) -> Result<Vec<Entry>, SourceError> {
		let mut entries = Vec::new();

		for s in &mut self.0 {
			entries.extend(match s {
				Kind::Http(x) => x.get().await?, // TODO: should HTTP even take a read filter?
				Kind::Twitter(x) => x.get().await?,
				Kind::File(x) => x.get().await?,
			});
		}

		Ok(entries)
	}

	// /// Mark an entry id as read if there's an rf available
	// #[allow(clippy::missing_errors_doc)]
	// pub async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
	// 	if let Some(rf) = self.rf.as_mut() {
	// 		rf.mark_as_read(id).await?;
	// 	}

	// 	Ok(())
	// }

	// /// Delegate for [`Source::remove_read`]
	// pub fn remove_read(&self, entries: &mut Vec<Entry>) {
	// 	if let Some(rf) = self.rf.as_ref() {
	// 		rf.remove_read_from(entries);
	// 	}
	// }
}

impl TryFrom<Vec<Kind>> for Source {
	type Error = SourceError;

	fn try_from(value: Vec<Kind>) -> Result<Self, Self::Error> {
		Self::new(value)
	}
}

impl std::ops::Deref for Source {
	type Target = [Kind];

	fn deref(&self) -> &Self::Target {
		self.0.as_slice()
	}
}

// TODO: can this be used to change the enum variant of a source kind?
impl std::ops::DerefMut for Source {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.as_mut_slice()
	}
}
