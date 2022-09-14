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
pub use self::field::shorten::Shorten;
pub use self::field::trim::Trim;

use self::field::Field;
use crate::{entry::Entry, error::transform::Error as TransformError, sink::Message};

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Transform {
	Entry(entry::Kind),
	Field { field: Field, kind: field::Kind },
}

impl Transform {
	pub async fn transform(&self, mut entry: Entry) -> Result<Vec<Entry>, TransformError> {
		match self {
			Self::Entry(ent_tr) => ent_tr.transform(entry).await,
			Self::Field { field, kind } => {
				let old_val = match field {
					Field::Title => entry.msg.title.take(),
					Field::Body => entry.msg.body.take(),
				};

				let new_val = old_val
					.as_deref()
					.map(|v| kind.transform_field(v))
					.transpose()
					.map_err(|kind| TransformError {
						kind,
						original_entry: entry.clone(),
					})?;

				let final_val = match new_val {
					None => old_val,
					Some(v) => v.get(old_val),
				};

				Ok(vec![match field {
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

impl<T: Into<entry::Kind>> From<T> for Transform {
	fn from(kind: T) -> Self {
		Self::Entry(kind.into())
	}
}
