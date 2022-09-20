/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`TransformEntry`](`entry::TransformEntry`) and [`TransformField`] traits as well as all types that implement it

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

use self::field::{Field, TransformField};
use crate::{entry::Entry, error::transform::Error as TransformError, sink::Message};

/// Either an entry or a field transform
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Transform {
	/// Transform entry
	Entry(entry::Kind),
	/// Transform [`field`]
	#[allow(missing_docs)]
	Field { field: Field, kind: field::Kind },
}

impl Transform {
	/// Transform [`entry`] with the current transform
	///
	/// # Errors
	/// if the inner transform errored out. Refer to its docs
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
