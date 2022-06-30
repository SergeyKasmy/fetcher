/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub mod html;
pub mod rss;

pub use self::html::Html;
pub use self::rss::Rss;

use crate::entry::Entry;
use crate::error::Result;

#[derive(Debug)]
pub enum Parser {
	Html(Html),
	Rss(Rss),
}

impl Parser {
	pub async fn parse(&self, entries: Vec<Entry>) -> Result<Vec<Entry>> {
		match self {
			Parser::Html(x) => x.parse(entries).await,
			Parser::Rss(x) => x.parse(entries),
		}
	}
}
