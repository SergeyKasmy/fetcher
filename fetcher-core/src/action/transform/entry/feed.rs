/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Feed`] transform that can parse RSS and Atom feeds

use super::TransformEntry;
use crate::action::transform::result::{
	TransformResult as TrRes, TransformedEntry, TransformedMessage,
};
use crate::entry::Entry;
use crate::error::transform::{FeedError, NothingToTransformError};

use url::Url;

/// RSS or Atom feed parser
#[derive(Debug)]
pub struct Feed;

impl TransformEntry for Feed {
	type Error = FeedError;

	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		tracing::debug!("Parsing feed entries");

		let feed = feed_rs::parser::parse(
			entry
				.raw_contents
				.as_ref()
				.ok_or(NothingToTransformError)?
				.as_bytes(),
		)
		.unwrap(); // TODO: check if feed is a valid feed

		tracing::debug!("Got {num} feed entries total", num = feed.entries.len());

		let entries = feed
			.entries
			.into_iter()
			.map(|mut feed_entry| {
				// unwrap NOTE: "safe", these are required fields	// TODO: make an error
				let id = Some(feed_entry.id);
				let title = Some(
					feed_entry
						.title
						.expect("RSS/Atom feeds should always contain a title")
						.content,
				);
				let body = Some(
					feed_entry
						.summary
						.expect("RSS/Atom feeds should always contain a summary/desciption")
						.content,
				);
				let link = Some(Url::try_from(feed_entry.links.remove(0).href.as_str()).unwrap()); // TODO: panics

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
