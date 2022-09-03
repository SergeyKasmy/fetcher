/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::entry::Entry;
use crate::error::transform::{FeedError, NothingToTransformError};
use crate::transform::result::{TransformResult as TrRes, TransformedEntry, TransformedMessage};

use url::Url;

#[derive(Debug)]
pub struct Feed;

impl Feed {
	#[tracing::instrument(skip_all)]
	pub fn transform(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, FeedError> {
		tracing::debug!("Parsing feed entries");

		let feed = feed_rs::parser::parse(
			entry
				.raw_contents
				.as_ref()
				.ok_or(NothingToTransformError)?
				.as_bytes(),
		)
		.unwrap();

		tracing::debug!("Got {num} feed entries total", num = feed.entries.len());

		let entries = feed
			.entries
			.into_iter()
			.map(|mut feed_entry| {
				// unwrap NOTE: "safe", these are required fields	// TODO: make an error
				let id = Some(feed_entry.id);
				let title = Some(feed_entry.title.unwrap().content);
				let body = Some(feed_entry.summary.unwrap().content);
				let link = Some(Url::try_from(feed_entry.links.remove(0).href.as_str()).unwrap()); // panics

				TransformedEntry {
					id: TrRes::New(id),
					raw_contents: TrRes::New(body.clone()),
					msg: TransformedMessage {
						title: TrRes::New(title),
						body: TrRes::New(body),
						link: TrRes::New(link),
						..Default::default()
					},
				}
			})
			.collect::<Vec<_>>();

		Ok(entries)
	}
}
