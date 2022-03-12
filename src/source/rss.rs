/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use rss::Channel;

use crate::entry::Entry;
use crate::error::Result;
use crate::read_filter::Id;
use crate::read_filter::ReadFilter;
use crate::sink::message::Link;
use crate::sink::message::LinkLocation;
use crate::sink::Message;

pub struct Rss {
	// TODO: use url
	url: String,
	http_client: reqwest::Client,
}

impl Rss {
	#[must_use]
	pub fn new(url: String) -> Self {
		Self {
			url,
			http_client: reqwest::Client::new(),
		}
	}

	#[tracing::instrument(skip_all)]
	pub async fn get(&mut self, read_filter: &ReadFilter) -> Result<Vec<Entry>> {
		tracing::debug!("Getting RSS articles");
		let content = self
			.http_client
			.get(&self.url)
			.send()
			.await?
			.bytes()
			.await?;

		let feed = Channel::read_from(&content[..])?;

		tracing::debug!("Got {num} RSS articles total", num = feed.items.len());

		let mut entries = feed
			.items
			.into_iter()
			.map(|x| {
				Entry {
					id: x.guid.as_ref().unwrap().value.clone(), // unwrap NOTE: same as above
					msg: Message {
						// unwrap NOTE: "safe", these are required fields
						title: Some(x.title.unwrap()),
						body: x.description.unwrap(),
						link: Some(Link {
							url: x.link.unwrap().as_str().try_into().unwrap(),
							loc: LinkLocation::PreferTitle,
						}), // unwrap FIXME: may be an invalid url
						media: None,
					},
				}
			})
			.collect();
		read_filter.remove_read_from(&mut entries);

		let unread_num = entries.len();
		if unread_num > 0 {
			tracing::info!("Got {unread_num} unread RSS articles");
		} else {
			tracing::debug!("All articles have already been read, none remaining to send");
		}

		entries.reverse();
		Ok(entries)
	}
}

impl std::fmt::Debug for Rss {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Rss")
			// .field("name", &self.name)
			.field("url", &self.url)
			.finish_non_exhaustive()
	}
}

impl Id for rss::Item {
	fn id(&self) -> &str {
		self.guid().unwrap().value()
	}
}
