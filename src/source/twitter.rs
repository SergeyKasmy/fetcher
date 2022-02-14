/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use egg_mode::entities::MediaType;
use egg_mode::{auth::bearer_token, tweet::user_timeline, KeyPair, Token};
use serde::Deserialize;

use crate::config;
use crate::error::{Error, Result};
use crate::sink::Media;
use crate::sink::Message;
use crate::source::Responce;

#[derive(Deserialize)]
#[serde(try_from = "config::Twitter")]
pub struct Twitter {
	pretty_name: String, // used for hashtags
	handle: String,
	api_key: String,
	api_secret: String,
	token: Option<Token>,
	filter: Vec<String>,
}

impl Twitter {
	#[tracing::instrument(skip(api_key, api_secret))]
	pub fn new(
		pretty_name: String,
		handle: String,
		api_key: String,
		api_secret: String,
		filter: Vec<String>,
	) -> Result<Self> {
		tracing::info!("Creatng a Twitter provider");
		Ok(Self {
			pretty_name,
			handle,
			api_key,
			api_secret,
			token: None,
			filter,
		})
	}

	#[tracing::instrument]
	pub async fn get(&mut self, last_read_id: Option<String>) -> Result<Vec<Responce>> {
		if self.token.is_none() {
			self.token = Some(
				bearer_token(&KeyPair::new(self.api_key.clone(), self.api_secret.clone()))
					.await
					.map_err(Error::TwitterAuth)?,
			);
		}
		// unwrap NOTE: initialized just above, should be safe
		let token = self.token.as_ref().unwrap();

		let (_, tweets) = user_timeline(self.handle.clone(), false, true, token) // TODO: remove clone
			.older(last_read_id.as_ref().and_then(|x| x.parse().ok()))
			.await?;

		// if !tweets.is_empty() {
		// 	tracing::info!(
		// 		"Got {amount} unread & unfiltered tweets",
		// 		amount = tweets.len()
		// 	);
		// }

		let messages = tweets
			.iter()
			.rev()
			.filter_map(|tweet| {
				if !self.filter.is_empty()
					&& !Self::tweet_contains_filters(&tweet.text, self.filter.as_slice())
				{
					return None;
				}

				let text = format!(
					"#{}\n\n{}\n<a href=\"https://twitter.com/{}/status/{}\">Link</a>",
					self.pretty_name, tweet.text, self.handle, tweet.id
				);

				Some(Responce {
					id: Some(tweet.id.to_string()),
					msg: Message {
						text,
						media: tweet.entities.media.as_ref().and_then(|x| {
							x.iter()
								.map(|x| match x.media_type {
									MediaType::Photo => {
										Some(Media::Photo(x.media_url.as_str().try_into().unwrap())) // unwrap NOTE: should be safe. If the string provided by tweeter is not an actual url, we should probably crash anyways
									}
									MediaType::Video => {
										Some(Media::Video(x.media_url.as_str().try_into().unwrap()))
									}
									MediaType::Gif => None,
								})
								.collect::<Option<Vec<Media>>>()
						}),
					},
				})
			})
			.collect::<Vec<_>>();

		Ok(messages)
	}

	fn tweet_contains_filters(tweet: &str, filters: &[String]) -> bool {
		for filter in filters {
			if !tweet.to_lowercase().contains(&filter.to_lowercase()) {
				return false;
			}
		}

		true
	}
}

impl std::fmt::Debug for Twitter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Twitter")
			.field("pretty_name", &self.pretty_name)
			.field("handle", &self.handle)
			.field("filter", &self.filter)
			.finish_non_exhaustive()
	}
}
