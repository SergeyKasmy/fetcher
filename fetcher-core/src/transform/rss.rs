/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rss::Channel;

use crate::entry::Entry;
use crate::error::transform::{NothingToTransformError, RssError};
use crate::sink::Message;

#[derive(Debug)]
pub struct Rss;

impl Rss {
	#[tracing::instrument(skip_all)]
	pub fn transform(&self, entry: &Entry) -> Result<Vec<Entry>, RssError> {
		tracing::debug!("Parsing RSS articles");

		let feed = Channel::read_from(
			entry
				.msg
				.body
				.as_ref()
				.ok_or(NothingToTransformError)?
				.as_bytes(),
		)?;

		tracing::debug!("Got {num} RSS articles total", num = feed.items.len());

		let entries = feed
			.items
			.into_iter()
			.map(|x| {
				Entry {
					id: Some(x.guid.as_ref().unwrap().value.clone()), // unwrap NOTE: same as above
					msg: Message {
						// unwrap NOTE: "safe", these are required fields
						title: Some(x.title.unwrap()),
						body: Some(x.description.unwrap()),
						link: Some(x.link.unwrap().as_str().try_into().unwrap()),
						..Default::default()
					},
					..Default::default()
				}
			})
			.collect::<Vec<_>>();

		Ok(entries)
	}
}
