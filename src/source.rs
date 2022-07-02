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
		match self {
			Source::WithSharedReadFilter(x) => x.get(parsers).await,
			Source::WithCustomReadFilter(x) => x.get(parsers).await,
		}
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
	#[must_use]
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

	pub async fn get(&mut self, parsers: Option<&[Parser]>) -> Result<Vec<Entry>> {
		let mut entries = Vec::new();

		for s in &mut self.sources {
			entries.extend(match s {
				WithSharedReadFilterInner::Http(x) => {
					let mut data = x.get().await?;
					if let Some(parsers) = parsers {
						for parser in parsers {
							data = parser.parse(data).await?;
						}
					}

					data
				}
				// TODO: Twitter source doesn't support parsing data
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
	pub async fn get(&mut self, parsers: Option<&[Parser]>) -> Result<Vec<Entry>> {
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
