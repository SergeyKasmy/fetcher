/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add google calendar source. Google OAuth2 is already implemented :)

pub mod with_custom_rf;
pub mod with_shared_rf;

pub mod parser;

pub use self::with_custom_rf::email::Email;
pub use self::with_shared_rf::file::File;
pub use self::with_shared_rf::http::Http;
pub use self::with_shared_rf::twitter::Twitter;

use itertools::Itertools;

use self::parser::Parser;
use crate::entry::Entry;
use crate::error::source::parse::Error as ParseError;
use crate::error::source::Error as SourceError;
use crate::error::Error;

#[derive(Debug)]
pub enum Source {
	WithSharedReadFilter(with_shared_rf::Source),
	WithCustomReadFilter(with_custom_rf::Source),
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
