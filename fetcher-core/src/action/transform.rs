/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod caps;
pub mod feed;
pub mod html;
pub mod json;
pub mod print;
pub mod regex;
pub mod result;
pub mod shorten;
// pub mod take;
pub mod trim;
pub mod use_raw_contents;

pub use self::caps::Caps;
pub use self::feed::Feed;
pub use self::html::Html;
pub use self::json::Json;
pub use self::regex::Regex;
pub use self::shorten::Shorten;
// pub use self::take::Take;
pub use self::trim::Trim;
pub use self::use_raw_contents::UseRawContents;

use self::result::TransformedEntry;
use crate::entry::Entry;
use crate::error::transform::Error as TransformError;
use crate::error::transform::Kind as TransformErrorKind;
use crate::sink::Message;
use crate::source::with_shared_rf::http;
use crate::source::Http;

trait TransformEntry {
	type Error: Into<TransformErrorKind>;

	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error>;
}

trait TransformField {
	fn transform_field(&self, field: &str) -> String;
}

#[derive(Debug)]
pub enum Kind {
	Entry(TransformEntryKind),
	Field(TransformFieldStruct),
}

#[derive(Debug)]
pub struct TransformFieldStruct {
	field: TransformFieldName,
	kind: TransformFieldKind,
}

#[derive(Debug)]
pub enum TransformFieldName {
	Title,
	Body,
}

#[derive(Debug)]
pub enum TransformFieldKind {
	Regex(Regex),
	Caps(Caps),
	Trim(Trim),
	Shorten(Shorten),
}

impl Kind {
	pub async fn transform(&self, mut entry: Entry) -> Result<Vec<Entry>, TransformError> {
		match self {
			Self::Entry(ent_tr) => ent_tr.transform(entry).await,
			Self::Field(field_tr) => {
				use TransformFieldKind::{Caps, Regex, Shorten, Trim};

				let field = match &field_tr.field {
					TransformFieldName::Title => entry.msg.title.take(),
					TransformFieldName::Body => entry.msg.body.take(),
				}
				.expect("TODO"); // TODO

				let field = match &field_tr.kind {
					Regex(tr) => tr.transform_field(&field),
					Caps(tr) => tr.transform_field(&field),
					Trim(tr) => tr.transform_field(&field),
					Shorten(tr) => tr.transform_field(&field),
				};

				Ok(vec![match &field_tr.field {
					TransformFieldName::Title => Entry {
						msg: Message {
							title: Some(field),
							..entry.msg
						},
						..entry
					},
					TransformFieldName::Body => Entry {
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

/// Type that allows transformation of a single [`Entry`] into one or multiple separate entries.
/// That includes everything from parsing a markdown format like JSON to simple transformations like making all text uppercase
// NOTE: Feed (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sizes but is trying to avoid that worth the hasle of a Box?
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum TransformEntryKind {
	Http,
	Html(Html),
	Json(Json),
	Feed(Feed),

	/// use [`raw_contents`](`crate::entry::Entry::raw_contents`) as message's [`body`](`crate::sink::Message::body`)
	UseRawContents(UseRawContents),
	Print,
}

impl TransformEntryKind {
	pub async fn transform(&self, entry: Entry) -> Result<Vec<Entry>, TransformError> {
		macro_rules! transform_delegate {
			($($t:tt),+ custom => { $($custom_t:pat => $custom_impl:expr),+ }) => {
				match self {
					$(Self::$t(x) => x.transform_entry(&entry).map_err(Into::into),)+
					$($custom_t => $custom_impl,)+
				}
			};
		}

		let res: Result<Vec<TransformedEntry>, TransformErrorKind> = transform_delegate!(
			Html, Json, Feed, UseRawContents

			custom => {
				Self::Http => {
					Http::transform(&entry, http::TransformFromField::MessageLink) // TODO: make this a choise
						.await
						.map(|x| vec![x])
						.map_err(Into::into)
				},
				Self::Print => {
					print::print(&entry).await;
					Ok(Vec::new())
				}
			}
		);

		res.map_err(|kind| TransformError {
			kind,
			original_entry: entry.clone(),
		})
		.map(|v| {
			if v.is_empty() {
				// pass-through the previous entry if the resulting after transforms vec is empty, i.e. if there was nothing done to the old entry
				vec![entry]
			} else {
				v.into_iter()
					.map(|new_entry| new_entry.into_entry(entry.clone()))
					.collect()
			}
		})
	}
}
