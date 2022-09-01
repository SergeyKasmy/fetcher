/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod caps;
pub mod html;
pub mod json;
pub mod print;
pub mod regex;
pub mod rss;
pub mod use_raw_contents;

pub use self::html::Html;
pub use self::json::Json;
pub use self::regex::Regex;
pub use self::rss::Rss;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::entry::Entry;
use crate::error::transform::Error as TransformError;
use crate::error::transform::Kind as TransformErrorKind;
use crate::read_filter::ReadFilter;
use crate::sink::Message;
use crate::source::with_shared_rf::http::TransformFromField;
use crate::source::Http;

/// Type that allows transformation of a single [`Entry`] into one or multiple separate entries.
/// That includes everything from parsing a markdown format like JSON to simple transformations like making all text uppercase
// NOTE: Rss (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sizes but is trying to avoid that worth the hasle of a Box?
// TODO: add raw_contents -> msg.body transformator
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Transform {
	// transform from one data type to another
	Http,
	Html(Html),
	Json(Json),
	Rss(Rss),

	// filter data
	ReadFilter(Arc<RwLock<ReadFilter>>),
	Regex(Regex),

	// modify data in-place
	/// use [`raw_contents`](`crate::entry::Entry::raw_contents`) as message's [`body`](`crate::sink::Message::body`)
	UseRawContents,
	Caps,

	// other
	Print,
}

impl Transform {
	/// Transform the entry `entry` into one or more entries
	///
	/// # Errors
	/// if there was an error parsing the entry
	pub async fn transform(&self, mut entries: Vec<Entry>) -> Result<Vec<Entry>, TransformError> {
		Ok(if let Transform::ReadFilter(rf) = self {
			rf.read().await.remove_read_from(&mut entries);
			entries
		} else {
			let mut fully_transformed_entries = Vec::new();
			for entry in entries {
				fully_transformed_entries.extend(self.transform_one(entry).await?);
			}

			fully_transformed_entries
		})
	}

	async fn transform_one(&self, entry: Entry) -> Result<Vec<Entry>, TransformError> {
		let res: Result<_, TransformErrorKind> = match self {
			Transform::Http => {
				Http::transform(&entry, TransformFromField::MessageLink) // TODO: make this a choise
					.await
					.map(|x| vec![x])
					.map_err(Into::into)
			}
			Transform::Html(x) => x.transform(&entry).map_err(Into::into),
			Transform::Json(x) => x.transform(&entry).map_err(Into::into),
			Transform::Rss(x) => x.transform(&entry).map_err(Into::into),
			// Transform::ReadFilter(rf) => Ok(rf.read().await.transform(&entries)),
			Transform::ReadFilter(_) => {
				unreachable!("Read filter doesn't support transforming one by one")
			}
			Transform::Regex(x) => x.transform(&entry).map(|x| vec![x]).map_err(Into::into),
			Transform::UseRawContents => Ok(vec![use_raw_contents::transform(&entry)]),
			Transform::Caps => Ok(vec![caps::transform(&entry)]),
			Transform::Print => {
				print::transform(&entry).await;
				Ok(vec![Entry::default()])
			}
		};

		res.map_err(|kind| TransformError {
			kind,
			original_entry: entry.clone(),
		})
		.map(|v| {
			// TODO: check if v is empty (mb make it an option even), and return the last entry if it is.
			// to avoid the unnecessary `Ok(vec![Entry::default()])` in the print transform
			v.into_iter()
				// use old entry's value if some new entry's field is None
				.map(|new_entry| Entry {
					id: new_entry.id.or_else(|| entry.id.clone()),
					raw_contents: new_entry
						.raw_contents
						.or_else(|| entry.raw_contents.clone()),
					msg: Message {
						title: new_entry.msg.title.or_else(|| entry.msg.title.clone()),
						body: new_entry.msg.body.or_else(|| entry.msg.body.clone()),
						link: new_entry.msg.link.or_else(|| entry.msg.link.clone()),
						media: new_entry.msg.media.or_else(|| entry.msg.media.clone()),
					},
				})
				.collect()
		})
	}
}
