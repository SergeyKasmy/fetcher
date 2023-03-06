/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Feed`] transform that can parse RSS and Atom feeds

use super::TransformEntry;
use crate::{
	action::transform::{
		error::RawContentsNotSetError,
		result::{TransformResult as TrRes, TransformedEntry, TransformedMessage},
	},
	entry::Entry,
};

use async_trait::async_trait;
use tap::{TapFallible, TapOptional};
use url::Url;

/// RSS or Atom feed parser
#[derive(Debug)]
pub struct Feed;

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum FeedError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error(transparent)]
	Other(#[from] feed_rs::parser::ParseFeedError),
}

#[async_trait]
impl TransformEntry for Feed {
	type Err = FeedError;

	async fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		tracing::trace!("Parsing feed entries");

		let feed = feed_rs::parser::parse(
			entry
				.raw_contents
				.as_ref()
				.ok_or(RawContentsNotSetError)?
				.as_bytes(),
		)?;

		tracing::debug!("Got {num} feed entries total", num = feed.entries.len());

		let entries = feed
			.entries
			.into_iter()
			.map(|mut feed_entry| {
				let title = feed_entry
					.title
					.tap_none(|| tracing::error!("Feed entry doesn't contain a title"))
					.map(|x| x.content);

				let body = feed_entry
					.summary
					.tap_none(|| {
						tracing::error!("Feed entry doesn't contain a summary/description/body");
					})
					.map(|x| x.content);

				let id = Some(feed_entry.id);

				let link = Url::try_from(feed_entry.links.remove(0).href.as_str())
					.tap_err(|e| tracing::warn!("A feed entry's link is not a valid URL: {e:?}"))
					.ok();

				TransformedEntry {
					id: TrRes::Old(id),
					raw_contents: TrRes::Old(body.clone()),
					msg: TransformedMessage {
						title: TrRes::Old(title),
						body: TrRes::Old(body),
						link: TrRes::Old(link),
						..Default::default()
					},
				}
			})
			.collect::<Vec<_>>();

		Ok(entries)
	}
}
