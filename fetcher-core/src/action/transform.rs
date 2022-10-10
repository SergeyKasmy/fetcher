/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`TransformEntry`](`entry::TransformEntry`) and [`TransformField`] traits as well as all types that implement it

pub mod entry;
pub mod field;
pub mod result;

use self::field::{Field, TransformField};
pub use self::{
	entry::{feed::Feed, html::Html, json::Json, use_raw_contents::UseRawContents},
	field::{caps::Caps, shorten::Shorten, trim::Trim},
};
use crate::utils::OptionExt;
use crate::{
	entry::Entry, error::transform::Error as TransformError, error::transform::InvalidUrlError,
	error::transform::Kind as TransformErrorKind, sink::Message,
};

use reqwest::Url;

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
	pub async fn transform(
		&self,
		mut entry: Entry,
		output: &mut Vec<Entry>,
	) -> Result<(), TransformError> {
		match self {
			Self::Entry(ent_tr) => ent_tr.transform(entry, output).await,
			Self::Field { field, kind } => {
				// old value of the field
				let old_val = match field {
					Field::Title => entry.msg.title.take(),
					Field::Body => entry.msg.body.take(),
					Field::Link => entry.msg.link.take().map(|u| u.to_string()),
				};

				let new_val =
					kind.transform_field(old_val.as_deref())
						.map_err(|kind| TransformError {
							kind,
							original_entry: entry.clone(),
						})?;

				// finalized value of the field. It's the new value that can get replaced with the old value if requested
				let final_val = new_val.get(old_val);

				let new_entry = match field {
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
					Field::Link => {
						let link = final_val.try_map(|s| {
							Url::try_from(s.as_str()).map_err(|e| TransformError {
								kind: TransformErrorKind::FieldLinkTransformInvalidUrl(
									InvalidUrlError(e, s),
								),
								original_entry: entry.clone(),
							})
						})?;

						Entry {
							msg: Message { link, ..entry.msg },
							..entry
						}
					}
				};

				output.push(new_entry);
				Ok(())
			}
		}
	}
}

impl<T: Into<entry::Kind>> From<T> for Transform {
	fn from(kind: T) -> Self {
		Self::Entry(kind.into())
	}
}
