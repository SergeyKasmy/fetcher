/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::source;

pub mod html;
pub mod rss;

use self::html::Html;

#[allow(clippy::large_enum_variant)] // this enum is very short-lived, I don't think boxing is worth the trouble
#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub(crate) enum Parser {
	Html(Html),
	Rss,

	Caps,
}

impl Parser {
	pub(crate) fn parse(self) -> source::parser::Parser {
		match self {
			Parser::Html(x) => source::parser::Parser::Html(x.parse()),
			Parser::Rss => source::parser::Parser::Rss(source::parser::Rss {}),

			Parser::Caps => source::parser::Parser::Caps(source::parser::Caps {}),
		}
	}
}
