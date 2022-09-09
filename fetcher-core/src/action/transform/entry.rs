/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod feed;
pub mod html;
pub mod json;
pub mod print;
pub mod use_raw_contents;

use self::feed::Feed;
use self::html::Html;
use self::json::Json;
use self::use_raw_contents::UseRawContents;
use super::result::TransformedEntry;
use crate::error::transform::Error as TransformError;
use crate::source::with_shared_rf::http;
use crate::source::Http;
use crate::{entry::Entry, error::transform::Kind as TransformErrorKind};

pub trait TransformEntry {
	type Error: Into<TransformErrorKind>;

	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error>;
}

/// Type that allows transformation of a single [`Entry`] into one or multiple separate entries.
/// That includes everything from parsing a markdown format like JSON to simple transformations like making all text uppercase
// NOTE: Feed (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sizes but is trying to avoid that worth the hasle of a Box?
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Kind {
	Http,
	Html(Html),
	Json(Json),
	Feed(Feed),

	/// use [`raw_contents`](`crate::entry::Entry::raw_contents`) as message's [`body`](`crate::sink::Message::body`)
	UseRawContents(UseRawContents),
	Print,
}

impl Kind {
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
