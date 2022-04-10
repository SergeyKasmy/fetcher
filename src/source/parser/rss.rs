/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use itertools::Itertools;
use rss::Channel;

use crate::entry::Entry;
use crate::error::Result;
use crate::sink::message::Link;
use crate::sink::message::LinkLocation;
use crate::sink::Message;

#[derive(Debug)]
pub struct Rss;

impl Rss {
	#[tracing::instrument(skip_all)]
	pub fn parse(&self, entries: Vec<Entry>) -> Result<Vec<Entry>> {
		tracing::debug!("Parsing RSS articles");

		entries
			.into_iter()
			.map(|entry| {
				let feed = Channel::read_from(entry.msg.body.as_bytes())?;

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
					.collect::<Vec<_>>();
				// read_filter.remove_read_from(&mut entries);	// FIXME

				let unread_num = entries.len();
				if unread_num > 0 {
					tracing::info!("Got {unread_num} unread RSS articles");
				} else {
					tracing::debug!("All articles have already been read, none remaining to send");
				}

				entries.reverse();
				Ok(entries)
			})
			.flatten_ok()
			.collect::<Result<Vec<Entry>>>()
	}
}
