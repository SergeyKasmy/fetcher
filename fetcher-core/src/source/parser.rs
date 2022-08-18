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
use crate::error::source::parse::Kind as ParseErrorKind;
use crate::sink::Message;

/// Type that allows transformation of a single [`Entry`] into one or multiple separate entries.
/// That includes everything from parsing a markdown format like JSON to simple transformations like making all text uppercase
// NOTE: Rss (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sizes but is trying to avoid that worth the hasle of a Box? TODO: Find out
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Parser {
	Html(Html),
	Json(Json),
	Rss(Rss),

	Caps(Caps),
}

impl Parser {
	/// Transform the entry `entry` into one or more entries
	///
	/// # Errors
	/// if there was an error parsing the entry
	pub fn parse(&self, mut entry: Entry) -> Result<Vec<Entry>, ParseError> {
		let res: Result<_, ParseErrorKind> = match self {
			Parser::Html(x) => x.parse(&entry).map_err(Into::into),
			Parser::Json(x) => x.parse(&entry).map_err(Into::into),
			Parser::Rss(x) => x.parse(&entry),

			Parser::Caps(x) => Ok(x.parse(&entry)),
		};

		res.map_err(|kind| ParseError {
			kind,
			original_entry: entry.clone(),
		})
		.map(|v| {
			v.into_iter()
				// use old entry's value if some new entry's field is None
				.map(|new_entry| Entry {
					id: new_entry.id.or_else(|| entry.id.take()),
					msg: Message {
						title: new_entry.msg.title.or_else(|| entry.msg.title.take()),
						body: new_entry.msg.body,
						link: new_entry.msg.link.or_else(|| entry.msg.link.take()),
						media: new_entry.msg.media.or_else(|| entry.msg.media.take()),
					},
				})
				.collect()
		})
	}
}
