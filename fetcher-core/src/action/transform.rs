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

				let old_val = match &field_tr.field {
					Field::Title => entry.msg.title.take(),
					Field::Body => entry.msg.body.take(),
				};

				let new_val = old_val.as_deref().map(|v| match &field_tr.kind {
					Regex(tr) => tr.transform_field(v),
					Caps(tr) => tr.transform_field(v),
					Trim(tr) => tr.transform_field(v),
					Shorten(tr) => tr.transform_field(v),
				});

				let final_val = match new_val {
					None => old_val,
					Some(v) => v.get(old_val),
				};

				Ok(vec![match &field_tr.field {
					Field::Title => Entry {
						msg: Message {
							title: final_val,
							..entry.msg
						},
						..entry
					},
					Field::Body => Entry {
						msg: Message {
							body: final_val,
							..entry.msg
						},
						..entry
					},
				}])
			}
		}
	}
}
