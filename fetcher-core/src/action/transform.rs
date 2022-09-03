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
use crate::source::with_shared_rf::http;
use crate::source::Http;

trait Transform {
	type Error: Into<TransformErrorKind>;

	fn transform(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error>;
}

/// Type that allows transformation of a single [`Entry`] into one or multiple separate entries.
/// That includes everything from parsing a markdown format like JSON to simple transformations like making all text uppercase
// NOTE: Feed (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sizes but is trying to avoid that worth the hasle of a Box?
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Kind {
	// transform from one data type to another
	Http,
	Html(Html),
	Json(Json),
	Feed(Feed),

	// filter data
	// ReadFilter(Arc<RwLock<ReadFilter>>),
	Regex(Regex),
	// Take(Take),

	// modify data in-place
	/// use [`raw_contents`](`crate::entry::Entry::raw_contents`) as message's [`body`](`crate::sink::Message::body`)
	UseRawContents(UseRawContents),
	Caps(Caps),
	Trim(Trim),
	Shorten(Shorten),

	// other
	Print,
}

impl Kind {
	/// Transform the entry `entry` into one or more entries
	///
	/// # Errors
	/// if there was an error parsing the entry
	// pub async fn transform(&self, mut entries: Vec<Entry>) -> Result<Vec<Entry>, TransformError> {
	// 	let res = match self {
	// 		Transform::ReadFilter(rf) => {
	// 			rf.read().await.remove_read_from(&mut entries);
	// 			entries
	// 		}
	// 		Transform::Take(take) => {
	// 			take.filter(&mut entries);
	// 			entries
	// 		}
	// 		_ => {
	// 		}
	// 	};
	// 	Ok(res)
	// }

	pub async fn transform(&self, entry: Entry) -> Result<Vec<Entry>, TransformError> {
		macro_rules! transform_delegate {
			($($t:tt),+ custom => { $($custom_t:pat => $custom_impl:expr),+ }) => {
				match self {
					$(Self::$t(x) => x.transform(&entry).map_err(Into::into),)+
					$($custom_t => $custom_impl,)+
				}
			};
		}

		let res: Result<Vec<TransformedEntry>, TransformErrorKind> = transform_delegate!(
			Html, Json, Feed, Regex, UseRawContents, Caps, Trim, Shorten

			custom => {
				Kind::Http => {
					Http::transform(&entry, http::TransformFromField::MessageLink) // TODO: make this a choise
						.await
						.map(|x| vec![x])
						.map_err(Into::into)
				},
				Kind::Print => {
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
