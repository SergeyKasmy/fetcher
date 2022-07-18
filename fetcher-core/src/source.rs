/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add google calendar source. Google OAuth2 is already implemented :)

pub mod email;
pub mod file;
pub mod http;
pub mod parser;
pub mod twitter;

pub use self::email::Email;
pub use self::file::File;
pub use self::http::Http;
pub use self::twitter::Twitter;

use itertools::Itertools;

use self::parser::Parser;
use crate::entry::Entry;
use crate::error::source::parse::Error as ParseError;
use crate::error::source::{EmailError, Error as SourceError};
use crate::error::Error;
use crate::read_filter::ReadFilter;

#[derive(Debug)]
pub enum Source {
	WithSharedReadFilter(WithSharedReadFilter),
	WithCustomReadFilter(WithCustomReadFilter),
}

impl Source {
	/// Get all available entries from the source and run them through the parsers
	///
	/// # Errors
	/// * if there was an error fetching from the source
	/// * if there was an error parsing the just fetched entries
	pub async fn get(&mut self, parsers: Option<&[Parser]>) -> Result<Vec<Entry>, SourceError> {
		let unparsed_entries = match self {
			Source::WithSharedReadFilter(x) => x.get().await?,
			Source::WithCustomReadFilter(x) => x.get().await?,
		};

		let mut parsed_entries = Vec::new();

		if let Some(parsers) = parsers {
			for entry in unparsed_entries {
				let mut entries_to_parse = vec![entry];
				for parser in parsers {
					entries_to_parse = entries_to_parse
						.into_iter()
						.map(|e| parser.parse(e))
						.flatten_ok()
						.collect::<Result<Vec<_>, ParseError>>()?;
				}

				parsed_entries.extend(entries_to_parse);
			}
		} else {
			parsed_entries = unparsed_entries;
		}

		let total_num = parsed_entries.len();
		self.remove_read(&mut parsed_entries);

		parsed_entries = parsed_entries
			.into_iter()
			.unique_by(|x| x.id.clone()) // TODO: I don't like this clone...
			.collect();

		let unread_num = parsed_entries.len();
		if total_num != unread_num {
			tracing::debug!(
				"Removed {read_num} read entries, {unread_num} remaining",
				read_num = total_num - unread_num
			);
		}

		Ok(parsed_entries)
	}

	/// Mark the id as read
	///
	/// # Errors
	/// if there was an error writing the id to the permanent storage
	pub async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		match self {
			Self::WithSharedReadFilter(x) => x.mark_as_read(id).await,
			Self::WithCustomReadFilter(x) => x.mark_as_read(id).await.map_err(Error::Source),
		}
	}

	/// Remove all read entries from the list of entries
	///
	/// Uses the id and the read filter to find and remove read entries
	pub fn remove_read(&self, entries: &mut Vec<Entry>) {
		match self {
			Source::WithSharedReadFilter(x) => x.remove_read(entries),
			Source::WithCustomReadFilter(x) => x.remove_read(entries),
		}
	}
}

#[derive(Debug)]
pub struct WithSharedReadFilter {
	read_filter: Option<ReadFilter>,
	sources: Vec<WithSharedReadFilterInner>,
}

#[derive(Debug)]
pub enum WithSharedReadFilterInner {
	File(File),
	Http(Http),
	Twitter(Twitter),
}

#[derive(Debug)]
pub enum WithCustomReadFilter {
	Email(Email),
}

impl WithSharedReadFilter {
	/// Create a new source struct that may contain one or several pure sources of the same type
	///
	/// # Errors
	/// * if the source list is empty
	/// * if the several sources that were provided are of different `WithStaredReadFilterInner` variants
	pub fn new(
		sources: Vec<WithSharedReadFilterInner>,
		read_filter: Option<ReadFilter>,
	) -> Result<Self, SourceError> {
		match sources.len() {
			0 => return Err(SourceError::EmptySourceList),
			1 => (),
			// assert that all source types are of the same enum variant
			_ => {
				// TODO: make a try_fold and shortcircuit of a different variant was found
				if !sources.windows(2).fold(true, |is_same, x| {
					if is_same {
						std::mem::discriminant(&x[0]) == std::mem::discriminant(&x[1])
					} else {
						is_same
					}
				}) {
					return Err(SourceError::SourceListHasDifferentVariants);
				}
			}
		}

		Ok(Self {
			read_filter,
			sources,
		})
	}

	/// Get all entries from the sources
	///
	/// # Errors
	/// if there was an error fetching from a source
	pub async fn get(&mut self) -> Result<Vec<Entry>, SourceError> {
		let mut entries = Vec::new();

		for s in &mut self.sources {
			entries.extend(match s {
				WithSharedReadFilterInner::Http(x) => x.get().await?, // TODO: should HTTP even take a read filter?
				WithSharedReadFilterInner::Twitter(x) => x.get(self.read_filter.as_ref()).await?,
				WithSharedReadFilterInner::File(x) => x.get().await?,
			});
		}

		Ok(entries)
	}

	/// Delegate for [`Source::mark_as_read`]
	#[allow(clippy::missing_errors_doc)]
	pub async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		if let Some(rf) = self.read_filter.as_mut() {
			rf.mark_as_read(id).await?;
		}

		Ok(())
	}

	/// Delegate for [`Source::remove_read`]
	pub fn remove_read(&self, entries: &mut Vec<Entry>) {
		if let Some(rf) = self.read_filter.as_ref() {
			rf.remove_read_from(entries);
		}
	}
}

impl WithCustomReadFilter {
	/// Fetch all entries from the source
	///
	/// # Errors
	/// if there was an error fetching from the source (such as a network connection error or maybe even an authentication error)
	pub async fn get(&mut self) -> Result<Vec<Entry>, SourceError> {
		Ok(match self {
			Self::Email(x) => x.get().await.map_err(Box::new)?,
		})
	}

	/// Delegate for [`Source::mark_as_read`]
	#[allow(clippy::missing_errors_doc)]
	pub async fn mark_as_read(&mut self, id: &str) -> Result<(), SourceError> {
		match self {
			Self::Email(x) => x
				.mark_as_read(id)
				.await
				.map_err(|e| Box::new(EmailError::Imap(e)))?,
		};

		Ok(())
	}

	/// Delegate for [`Source::remove_read`]
	#[allow(clippy::ptr_arg)]
	pub fn remove_read(&self, _entries: &mut Vec<Entry>) {
		match self {
			Self::Email(_) => (), // NO-OP, emails should already be unread only when fetching
		}
	}
}
