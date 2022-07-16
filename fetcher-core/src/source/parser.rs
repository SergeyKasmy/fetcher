/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod caps;
pub mod html;
pub mod json;
pub mod rss;

pub use self::caps::Caps;
pub use self::html::Html;
pub use self::json::Json;
pub use self::rss::Rss;

use crate::entry::Entry;
use crate::error::source::parse::Error as ParseError;

// NOTE: Rss (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sices but is trying to avoid that worth the hasle of a Box? TODO: Find out
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Parser {
	Html(Html),
	Json(Json),
	Rss(Rss),

	Caps(Caps),
}

impl Parser {
	pub fn parse(&self, entry: Entry) -> Result<Vec<Entry>, ParseError> {
		Ok(match self {
			Parser::Html(x) => x.parse(entry)?,
			Parser::Json(x) => x.parse(entry)?,
			Parser::Rss(x) => x.parse(entry)?,

			Parser::Caps(x) => x.parse(entry),
		})
	}
}
