/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: add google calendar source. Google OAuth2 is already implemented :)

pub mod email;
pub mod http;
pub mod parser;
pub mod twitter;

use itertools::Itertools;

pub use self::email::Email;
pub use self::http::Http;
use self::parser::Parser;
pub use self::twitter::Twitter;

use crate::entry::Entry;
use crate::error::{Error, Result};
use crate::read_filter::ReadFilter;

#[derive(Debug)]
pub enum Source {
	WithSharedReadFilter(WithSharedReadFilter),
	WithCustomReadFilter(WithCustomReadFilter),
}

impl Source {
	pub async fn get(&mut self, parsers: Option<&[Parser]>) -> Result<Vec<Entry>> {
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
						.collect::<Result<Vec<_>>>()?;
				}

				parsed_entries.extend(entries_to_parse);
			}
		} else {
			parsed_entries = unparsed_entries;
		}

		Ok(parsed_entries)
	}

	pub async fn mark_as_read(&mut self, id: &str) -> Result<()> {
		match self {
			Self::WithSharedReadFilter(x) => x.mark_as_read(id).await,
			Self::WithCustomReadFilter(x) => x.mark_as_read(id).await,
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
	Http(Http),
	Twitter(Twitter),
}

#[derive(Debug)]
pub enum WithCustomReadFilter {
	Email(Email),
}

impl WithSharedReadFilter {
	pub fn new(sources: Vec<WithSharedReadFilterInner>, read_filter: ReadFilter) -> Result<Self> {
		match sources.len() {
			0 => {
				return Err(Error::IncompatibleConfigValues(
					"A task can't have 0 sources (path is not applicable)",
					std::path::PathBuf::new(),
				))
			}
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

	pub async fn get(&mut self) -> Result<Vec<Entry>> {
		let mut entries = Vec::new();

		for s in &mut self.sources {
			entries.extend(match s {
				WithSharedReadFilterInner::Http(x) => x.get().await?,
				WithSharedReadFilterInner::Twitter(x) => x.get(&self.read_filter).await?,
			});
		}

		Ok(entries)
	}

	pub async fn mark_as_read(&mut self, id: &str) -> Result<()> {
		self.read_filter.mark_as_read(id).await
	}
}

impl WithCustomReadFilter {
	pub async fn get(&mut self) -> Result<Vec<Entry>> {
		Ok(match self {
			Self::Email(x) => x.get().await?,
		})
	}

	pub async fn mark_as_read(&mut self, id: &str) -> Result<()> {
		match self {
			Self::Email(x) => x.mark_as_read(id).await,
		}
	}
}
