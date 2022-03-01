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

use serde::Deserialize;

use crate::error::Result;
use crate::sink::Message;

// TODO: add message history via responce id -> message id hashmap
// TODO: add pretty name/hashtag and link here instead of doing it manually
#[derive(Debug)]
pub struct Responce {
	pub id: Option<String>,
	pub msg: Message,
}

// TODO: add google calendar source. Google OAuth2 is already implemented :)
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
	Email(Email),
	Html(Html),
	Rss(Rss),
	Twitter(Twitter),
}

impl Source {
	// TODO: try using streams instead of polling manually?
	pub async fn get(&mut self, last_read_id: Option<String>) -> Result<Vec<Responce>> {
		match self {
			Self::Email(x) => x.get().await,
			Self::Html(x) => x.get(last_read_id).await,
			Self::Rss(x) => x.get(last_read_id).await,
			Self::Twitter(x) => x.get(last_read_id).await,
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
