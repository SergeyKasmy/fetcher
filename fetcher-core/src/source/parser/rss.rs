/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rss::Channel;

use crate::entry::Entry;
use crate::error::source::parse::Error as ParseError;
use crate::sink::Message;

#[derive(Debug)]
pub struct Rss;

impl Rss {
	#[tracing::instrument(skip_all)]
	pub fn parse(&self, entry: Entry) -> Result<Vec<Entry>, ParseError> {
		tracing::debug!("Parsing RSS articles");

		let feed = Channel::read_from(entry.msg.body.as_bytes())?;

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
						body: x.description.unwrap(),
						link: Some(x.link.unwrap().as_str().try_into().unwrap()),
						media: None,
					},
				}
			})
			.collect::<Vec<_>>();

		Ok(entries)
	}
}
