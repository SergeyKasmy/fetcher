/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use rss::Channel;
use serde::Deserialize;

use crate::error::Result;
use crate::sink::Message;
use crate::source::Responce;

#[derive(Deserialize)]
pub struct Rss {
	// name: String,
	url: String,
	#[serde(skip)]
	http_client: reqwest::Client,
}

impl Rss {
	#[tracing::instrument]
	pub fn new(/* name: String, */ url: String) -> Self {
		tracing::info!("Creatng an Rss provider");
		Self {
			// name,
			url,
			http_client: reqwest::Client::new(),
		}
	}

	#[tracing::instrument]
	pub async fn get(&mut self, last_read_id: Option<String>) -> Result<Vec<Responce>> {
		let content = self
			.http_client
			.get(&self.url)
			.send()
			.await?
			.bytes()
			.await?;

		let mut feed = Channel::read_from(&content[..])?;

		if let Some(id) = &last_read_id {
			if let Some(id_pos) = feed
				.items
				.iter()
				// unwrap NOTE: *should* be safe, rss without guid is kinda useless
				.position(|x| x.guid.as_ref().unwrap().value == id.as_str())
			{
				feed.items.drain(id_pos..);
			}
		}

		if !feed.items.is_empty() {
			tracing::info!(
				"Got {amount} unread RSS articles",
				amount = feed.items.len()
			);
		}

		let messages = feed
			.items
			.into_iter()
			.rev()
			.map(|x| {
				let text = format!(
					"<a href=\"{}\">{}</a>\n{}",
					// unwrap NOTE: "safe", these are required fields
					x.link.as_deref().unwrap(),
					x.title.as_deref().unwrap(),
					x.description.as_deref().unwrap()
				);

				Responce {
					id: Some(x.guid.as_ref().unwrap().value.clone()), // unwrap NOTE: same as above
					msg: Message { text, media: None },
				}
			})
			.collect();

		Ok(messages)
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
