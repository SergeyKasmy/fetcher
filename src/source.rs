/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub mod email;
pub mod html;
pub mod rss;
pub mod twitter;

pub use self::email::Email;
pub use self::html::Html;
pub use self::rss::Rss;
pub use self::twitter::Twitter;

use crate::entry::Entry;
use crate::error::Result;
use crate::read_filter::ReadFilter;

// TODO: add google calendar source. Google OAuth2 is already implemented :)
#[derive(Debug)]
pub enum Source {
	Email(Email),
	Html(Html),
	Rss(Rss),
	Twitter(Twitter),
}

impl Source {
	// TODO: try using streams instead of polling manually?
	#[allow(clippy::missing_errors_doc)] // TODO
	pub async fn get(&mut self, read_filter: Option<&ReadFilter>) -> Result<Vec<Entry>> {
		match self {
			Self::Email(x) => x.get().await,
			Self::Html(x) => x.get(read_filter.unwrap()).await,
			Self::Rss(x) => x.get(read_filter.unwrap()).await,
			Self::Twitter(x) => x.get(read_filter.unwrap()).await,
		}
	}
}

impl From<Email> for Source {
	fn from(e: Email) -> Self {
		Self::Email(e)
	}
}

impl From<Rss> for Source {
	fn from(r: Rss) -> Self {
		Self::Rss(r)
	}
}

impl From<Twitter> for Source {
	fn from(t: Twitter) -> Self {
		Self::Twitter(t)
	}
}
