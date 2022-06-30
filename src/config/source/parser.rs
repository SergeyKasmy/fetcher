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

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Parser {
	Rss,
	Html(Html),
}

impl Parser {
	pub(crate) fn parse(self) -> source::parser::Parser {
		match self {
			Parser::Rss => todo!(),
			Parser::Html(x) => source::parser::Parser::Html(x.parse()),
		}
	}
}
