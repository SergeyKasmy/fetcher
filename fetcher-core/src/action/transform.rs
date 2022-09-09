/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod entry;
pub mod field;
pub mod result;

pub use self::entry::feed::Feed;
pub use self::entry::html::Html;
pub use self::entry::json::Json;
pub use self::entry::use_raw_contents::UseRawContents;
pub use self::field::caps::Caps;
pub use self::field::regex::Regex;
pub use self::field::shorten::Shorten;
pub use self::field::trim::Trim;

use self::field::TransformField;
use crate::entry::Entry;
use crate::error::transform::Error as TransformError;
use crate::sink::Message;

#[derive(Debug)]
pub enum Kind {
	Entry(entry::Kind),
	Field(field::Transform),
}

impl Kind {
	pub async fn transform(&self, mut entry: Entry) -> Result<Vec<Entry>, TransformError> {
		match self {
			Self::Entry(ent_tr) => ent_tr.transform(entry).await,
			Self::Field(field_tr) => {
				use field::Field;
				use field::Kind::{Caps, Regex, Shorten, Trim};

				let field = match &field_tr.field {
					Field::Title => entry.msg.title.take(),
					Field::Body => entry.msg.body.take(),
				}
				.expect("TODO"); // TODO

				let field = match &field_tr.kind {
					Regex(tr) => tr.transform_field(&field),
					Caps(tr) => tr.transform_field(&field),
					Trim(tr) => tr.transform_field(&field),
					Shorten(tr) => tr.transform_field(&field),
				};

				Ok(vec![match &field_tr.field {
					Field::Title => Entry {
						msg: Message {
							title: Some(field),
							..entry.msg
						},
						..entry
					},
					Field::Body => Entry {
						msg: Message {
							body: Some(field),
							..entry.msg
						},
						..entry
					},
				}])
			}
		}
	}
}
