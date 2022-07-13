/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
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
use crate::error::source::Error as SourceError;
use crate::error::Error;
use crate::read_filter::ReadFilter;

#[derive(Debug)]
pub enum Source {
	WithSharedReadFilter(WithSharedReadFilter),
	WithCustomReadFilter(WithCustomReadFilter),
}

impl Source {
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
		match self {
			Source::WithSharedReadFilter(x) => x.remove_read(&mut parsed_entries),
			Source::WithCustomReadFilter(x) => x.remove_read(&mut parsed_entries),
		}

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

	pub async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		match self {
			Self::WithSharedReadFilter(x) => x.mark_as_read(id).await,
			Self::WithCustomReadFilter(x) => x.mark_as_read(id).await.map_err(Error::Source),
		}
	}
}

#[derive(Debug)]
pub struct WithSharedReadFilter {
	read_filter: ReadFilter,
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
	pub fn new(
		sources: Vec<WithSharedReadFilterInner>,
		read_filter: ReadFilter,
	) -> Result<Self, SourceError> {
		match sources.len() {
			0 => return Err(SourceError::EmptySourceList),
			1 => (),
			// assert that all source types are of the same enum variant
			_ => {
				assert!(sources.windows(2).fold(true, |is_same, x| {
					if is_same {
						std::mem::discriminant(&x[0]) == std::mem::discriminant(&x[1])
					} else {
						is_same
					}
				}));
			}
		}

		Ok(Self {
			read_filter,
			sources,
		})
	}

	pub async fn get(&mut self) -> Result<Vec<Entry>, SourceError> {
		let mut entries = Vec::new();

		for s in &mut self.sources {
			entries.extend(match s {
				WithSharedReadFilterInner::Http(x) => x.get().await?, // TODO: should HTTP even take a read filter?
				WithSharedReadFilterInner::Twitter(x) => x.get(&self.read_filter).await?,
				WithSharedReadFilterInner::File(x) => x.get().await?,
			});
		}

		Ok(entries)
	}

	pub async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		self.read_filter.mark_as_read(id).await
	}

	pub fn remove_read(&self, entries: &mut Vec<Entry>) {
		self.read_filter.remove_read_from(entries);
	}
}

impl WithCustomReadFilter {
	pub async fn get(&mut self) -> Result<Vec<Entry>, SourceError> {
		Ok(match self {
			Self::Email(x) => x.get().await?,
		})
	}

	pub async fn mark_as_read(&mut self, id: &str) -> Result<(), SourceError> {
		Ok(match self {
			Self::Email(x) => x
				.mark_as_read(id)
				.await
				.map_err(crate::error::source::EmailError::Imap)?,
		})
	}

	#[allow(clippy::ptr_arg)]
	pub fn remove_read(&self, _entries: &mut Vec<Entry>) {
		match self {
			Self::Email(_) => (), // NO-OP, emails should already be unread only when fetching
		}
	}
}
