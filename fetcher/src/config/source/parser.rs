/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod html;
pub mod json;

use serde::{Deserialize, Serialize};

use self::html::Html;
use self::json::Json;
use fetcher_core::source;

#[allow(clippy::large_enum_variant)] // this enum is very short-lived, I don't think boxing is worth the trouble
#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub(crate) enum Parser {
	Http,
	Html(Html),
	Json(Json),
	Rss,

	Caps,
}

impl Parser {
	pub(crate) fn parse(self) -> source::parser::Parser {
		match self {
			Parser::Http => source::parser::Parser::Http,
			Parser::Html(x) => source::parser::Parser::Html(x.parse()),
			Parser::Json(x) => source::parser::Parser::Json(x.parse()),
			Parser::Rss => source::parser::Parser::Rss(source::parser::Rss {}),

			Parser::Caps => source::parser::Parser::Caps(source::parser::Caps {}),
		}
	}
}
