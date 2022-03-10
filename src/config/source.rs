/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub(crate) mod email;
pub(crate) mod html;
pub(crate) mod rss;
pub(crate) mod twitter;

use serde::Deserialize;

use crate::error::Result;
use crate::source;

use self::email::Email;
use self::html::Html;
use self::rss::Rss;
use self::twitter::Twitter;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Source {
	Email(Email),
	Html(Html),
	Rss(Rss),
	Twitter(Twitter),
}

impl Source {
	pub(crate) fn parse(self) -> Result<source::Source> {
		Ok(match self {
			Source::Email(x) => source::Source::Email(x.parse()?),
			Source::Html(x) => source::Source::Html(x.parse()),
			Source::Rss(x) => source::Source::Rss(x.parse()),
			Source::Twitter(x) => source::Source::Twitter(x.parse()?),
		})
	}
}
