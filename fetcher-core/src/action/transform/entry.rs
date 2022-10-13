/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`TransformEntry`] trait as well as every type that implement it

pub mod feed;
pub mod html;
pub mod json;
pub mod print;
pub mod use_as;

use self::{feed::Feed, html::Html, json::Json, use_as::Use};
use super::result::TransformedEntry;
use crate::{
	entry::Entry,
	error::transform::{Error as TransformError, Kind as TransformErrorKind},
	source::http::{self, Http},
};

use derive_more::From;

/// A helper trait for transforms that transform a single entry into one or several separate entries
pub trait TransformEntry {
	/// Error return type. May be [`Infallible`](`std::convert::Infallible`)
	type Error: Into<TransformErrorKind>;

	/// Transform the `entry` into one or several separate entries
	#[allow(clippy::missing_errors_doc)]
	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error>;
}

/// Type that includes all available transforms that implement the [`TransformEntry`] trait.
/// That includes everything from parsing a markdown format like JSON to just debug printing
// NOTE: Feed (and probs others in the future) is a ZST, so there's always going to be some amount of variance of enum sizes but is trying to avoid that worth the hasle of a Box?
#[allow(missing_docs, clippy::large_enum_variant)]
#[derive(From, Debug)]
pub enum Kind {
	Http,
	Html(Html),
	Json(Json),
	Feed(Feed),

	/// use the contents of a field as a different field
	Use(Use),
	Print,
}

impl Kind {
	/// Calls each enum variant's [`transform_entry()`](`TransformEntry::transform_entry()`) impl
	/// # Errors
	/// for the same reason each of them may error. Refer to their individual docs
	// This type doesn't implement TransformEntry trait itself since the Http impl of that requires an async function
	// and there's no reason to add the overhead of a Box'ed future type (via #[async_trait]) just for that one impl.
	// If more transforms will require async in the future, I may as well make TransformEntry async and implement it for Kind
	pub async fn transform(
		&self,
		entry: Entry,
		output: &mut Vec<Entry>,
	) -> Result<(), TransformError> {
		macro_rules! delegate {
			($($t:tt),+ custom => { $($custom_t:pat => $custom_impl:expr),+ }) => {
				match self {
					$(Self::$t(x) => x.transform_entry(&entry).map_err(Into::into),)+
					$($custom_t => $custom_impl,)+
				}
			};
		}

		let v = delegate!(
			Html, Json, Feed, Use

			custom => {
				Self::Http => {
					Http::transform(&entry, http::TransformFromField::MessageLink) // TODO: make this a choise
						.await
						.map(|x| vec![x])
						.map_err(Into::into)
				},
				Self::Print => {
					print::print(&entry).await;
					Ok(vec![TransformedEntry::default()])
				}
			}
		)
		.map_err(|kind| TransformError {
			kind,
			original_entry: entry.clone(),
		})?;

		output.extend(
			v.into_iter()
				.map(|new_entry| new_entry.into_entry(entry.clone())),
		);

		Ok(())
	}
}
