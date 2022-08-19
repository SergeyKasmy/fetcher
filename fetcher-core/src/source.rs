/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add google calendar source. Google OAuth2 is already implemented :)

pub mod with_custom_rf;
pub mod with_shared_rf;

pub use self::with_custom_rf::email::Email;
pub use self::with_shared_rf::file::File;
pub use self::with_shared_rf::http::Http;
pub use self::with_shared_rf::twitter::Twitter;

use itertools::Itertools;

use crate::entry::Entry;
use crate::error::source::Error as SourceError;
use crate::error::Error;
use crate::transform::Transform;

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
	pub async fn get(
		&mut self,
		transforms: Option<&[Transform]>,
	) -> Result<Vec<Entry>, SourceError> {
		let unparsed_entries = match self {
			Source::WithSharedReadFilter(x) => x.get().await?,
			Source::WithCustomReadFilter(x) => x.get().await?,
		};

		let mut fully_parsed_entries = Vec::new(); // parsed with all parsers

		if let Some(transforms) = transforms {
			for entry in unparsed_entries {
				let mut entries_to_parse = vec![entry];
				for parser in transforms {
					let mut partially_parsed_entries = Vec::new(); // parsed only with the current parser
					for entry_to_parse in entries_to_parse {
						partially_parsed_entries
							.extend(parser.transform(entry_to_parse).await.unwrap()); // FIXME
					}
					entries_to_parse = partially_parsed_entries;
				}

				fully_parsed_entries.extend(entries_to_parse);
			}
		} else {
			fully_parsed_entries = unparsed_entries;
		}

		let total_num = fully_parsed_entries.len();
		self.remove_read(&mut fully_parsed_entries);

		fully_parsed_entries = fully_parsed_entries
			.into_iter()
			// TODO: I don't like this clone...
			// FIXME: removes all entries with no/empty id because "" == "". Maybe move to .remove_read()?
			.unique_by(|ent| ent.id.clone())
			.collect();

		let unread_num = fully_parsed_entries.len();
		if total_num != unread_num {
			tracing::debug!(
				"Removed {read_num} read entries, {unread_num} remaining",
				read_num = total_num - unread_num
			);
		}

		Ok(fully_parsed_entries)
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
