/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Feed`] transform that can parse RSS and Atom feeds

use super::Transform;
use crate::{
	actions::transforms::{
		error::RawContentsNotSetError,
		result::{OptionUnwrapTransformResultExt, TransformedEntry, TransformedMessage},
	},
	entry::Entry,
};

use feed_rs::model::{Content, Text};
use tap::TapOptional;

/// RSS or Atom feed parser
#[derive(Debug)]
pub struct Feed;

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum FeedError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error(transparent)]
	Other(#[from] feed_rs::parser::ParseFeedError),
}

impl Transform for Feed {
	type Err = FeedError;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
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
			.map(parse_feed_entry)
			.collect::<Vec<_>>();

		Ok(entries)
	}
}

fn parse_feed_entry(mut feed_entry: feed_rs::model::Entry) -> TransformedEntry {
	let title = feed_entry
		.title
		.tap_none(|| tracing::error!("Feed entry doesn't contain a title"))
		.map(|x| x.content);

	let body = message_body_from_feed_entry(feed_entry.summary, feed_entry.content);

	let id = Some(feed_entry.id);
	let link = Some(feed_entry.links.swap_remove(0).href);

	TransformedEntry {
		id: id.map(Into::into).unwrap_or_prev(),
		raw_contents: body.clone().unwrap_or_prev(),
		msg: TransformedMessage {
			title: title.unwrap_or_prev(),
			body: body.unwrap_or_prev(),
			link: link.unwrap_or_prev(),
			..Default::default()
		},
		..Default::default()
	}
}

#[expect(
	clippy::cognitive_complexity,
	reason = "simplified as much as necessary"
)]
fn message_body_from_feed_entry(summary: Option<Text>, content: Option<Content>) -> Option<String> {
	match summary.map(|text| text.content) {
		Some(summary) => {
			tracing::trace!(
				"Using the summary as the body of the message: {:?}{}",
				&summary[..100],
				if summary.len() > 100 { "..." } else { "" },
			);
			Some(summary)
		}
		None => {
			let content = content.and_then(|content| content.body);
			match content {
				Some(content) => {
					tracing::trace!(
						r#"Summary missing, falling back to "content": {:?}{}"#,
						&content[..100],
						if content.len() > 100 { "..." } else { "" },
					);
					Some(content)
				}
				None => {
					tracing::error!(
						"Feed entry doesn't contain a summary and doesn't have any contents"
					);
					None
				}
			}
		}
	}
}
