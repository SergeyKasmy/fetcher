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
pub mod take;
pub mod trim;
pub mod use_raw_contents;

pub use self::feed::Feed;
pub use self::html::Html;
pub use self::json::Json;
pub use self::regex::Regex;
pub use self::shorten::Shorten;
pub use self::take::Take;
pub use self::trim::Trim;

use self::result::TransformedEntry;
use crate::entry::Entry;
use crate::error::transform::Error as TransformError;
use crate::error::transform::Kind as TransformErrorKind;
use crate::read_filter::ReadFilter;
use crate::source::with_shared_rf::http::TransformFromField;
use crate::source::Http;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Type that allows transformation of a single [`Entry`] into one or multiple separate entries.
/// That includes everything from parsing a markdown format like JSON to simple transformations like making all text uppercase
// NOTE: Feed (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sizes but is trying to avoid that worth the hasle of a Box?
// TODO: add raw_contents -> msg.body transformator
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Transform {
	// transform from one data type to another
	Http,
	Html(Html),
	Json(Json),
	Feed(Feed),

	// filter data
	ReadFilter(Arc<RwLock<ReadFilter>>),
	Regex(Regex),
	Take(Take),

	// modify data in-place
	/// use [`raw_contents`](`crate::entry::Entry::raw_contents`) as message's [`body`](`crate::sink::Message::body`)
	UseRawContents,
	Caps,
	Trim(Trim),
	Shorten(Shorten),

	// other
	Print,
}

impl Transform {
	/// Transform the entry `entry` into one or more entries
	///
	/// # Errors
	/// if there was an error parsing the entry
	pub async fn transform(&self, mut entries: Vec<Entry>) -> Result<Vec<Entry>, TransformError> {
		let res = match self {
			Transform::ReadFilter(rf) => {
				rf.read().await.remove_read_from(&mut entries);
				entries
			}
			Transform::Take(take) => {
				take.filter(&mut entries);
				entries
			}
			_ => {
				let mut fully_transformed_entries = Vec::new();
				for entry in entries {
					fully_transformed_entries.extend(self.transform_one(entry).await?);
				}

				fully_transformed_entries
			}
		};
		Ok(res)
	}

	async fn transform_one(&self, entry: Entry) -> Result<Vec<Entry>, TransformError> {
		let res: Result<Vec<TransformedEntry>, TransformErrorKind> = match self {
			Transform::Http => {
				Http::transform(&entry, TransformFromField::MessageLink) // TODO: make this a choise
					.await
					.map(|x| vec![x])
					.map_err(Into::into)
			}
			Transform::Html(x) => x.transform(&entry).map_err(Into::into),
			Transform::Json(x) => x.transform(&entry).map_err(Into::into),
			Transform::Feed(x) => x.transform(&entry).map_err(Into::into),
			// Transform::ReadFilter(rf) => Ok(rf.read().await.transform(&entries)),
			Transform::ReadFilter(_) => {
				unreachable!("Read filter doesn't support transforming one by one")
			}
			Transform::Regex(x) => x.transform(&entry).map(|x| vec![x]).map_err(Into::into),
			Transform::Take(_) => {
				unreachable!("Take doesn't support transforming one by one")
			}
			Transform::UseRawContents => Ok(vec![use_raw_contents::transform(&entry)]),
			Transform::Caps => Ok(vec![caps::transform(&entry)]),
			Transform::Trim(x) => Ok(vec![x.transform(&entry)]),
			Transform::Shorten(x) => Ok(vec![x.transform(&entry)]),
			Transform::Print => {
				print::transform(&entry).await;
				Ok(vec![TransformedEntry::default()])
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
				.map(|new_entry| new_entry.into_entry(entry.clone()))
				.collect()
		})
	}
}
