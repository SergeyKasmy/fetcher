/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Twitter feed
//!
//! This module includes the [`Twitter`] struct that is a source that is able to parse a twitter feed via twitter API

use super::{Fetch, error::SourceError};
use crate::{
	entry::Entry,
	sink::message::{Media, Message},
};

use async_trait::async_trait;
use egg_mode::{
	KeyPair, Token,
	auth::bearer_token,
	entities::MediaType,
	tweet::{Timeline, user_timeline},
};

/// A source that fetches from a Twitter feed using the Twitter API
pub struct Twitter {
	// the only point of this option is to enable taking timeline by value. It can be never observed to be None unless the thread panicked
	timeline: Option<Timeline>,
	handle: String,
	auth: Auth,
}

enum Auth {
	NotAuthenticated { api_key: String, api_secret: String },
	Authenticated(Token),
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum TwitterError {
	#[error("Authentication failed")]
	Auth(#[source] egg_mode::error::Error),

	#[error(transparent)]
	Other(#[from] egg_mode::error::Error),
}

impl Twitter {
	/// Creates a new [`Twitter`] source
	#[must_use]
	pub const fn new(handle: String, api_key: String, api_secret: String) -> Self {
		Self {
			timeline: None,
			handle,
			auth: Auth::NotAuthenticated {
				api_key,
				api_secret,
			},
		}
	}
}

#[async_trait]
impl Fetch for Twitter {
	/// Fetches all tweets from the feed
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		self.fetch_impl().await.map_err(Into::into)
	}
}

impl Twitter {
	async fn fetch_impl(&mut self) -> Result<Vec<Entry>, TwitterError> {
		tracing::trace!("Getting tweets");

		let token = match &self.auth {
			Auth::NotAuthenticated {
				api_key,
				api_secret,
			} => {
				tracing::trace!("Not authenticated yet, authenticating...");
				let token = bearer_token(&KeyPair::new(api_key.clone(), api_secret.clone()))
					.await
					.map_err(TwitterError::Auth)?;

				self.auth = Auth::Authenticated(token);
				let Auth::Authenticated(auth) = &self.auth else {
					unreachable!("it has just been put there, this couldn't happen");
				};

				auth
			}
			Auth::Authenticated(token) => token,
		};

		let (timeline, tweets) = match &self.timeline {
			None => {
				tracing::trace!("Creating a new timeline/first time getting tweets"); // read filter should remove ones already read though
				user_timeline(self.handle.clone(), true, true, token)
					.start()
					.await?
			}
			Some(_) => {
				tracing::trace!(
					"Re-using timeline, just getting tweets newer than last time fetched"
				);
				self.timeline
					.take()
					.expect("shouldn't be None, just matched Some")
					.newer(None)
					.await?
			}
		};

		self.timeline = Some(timeline);

		tracing::debug!(
			"Got {num} tweets from timeline in total",
			num = tweets.len()
		);

		let messages = tweets
			.iter()
			.map(|tweet| Entry {
				id: Some(tweet.id.to_string().into()),
				reply_to: tweet.in_reply_to_status_id.map(|i| i.to_string().into()),
				msg: Message {
					body: Some(tweet.text.clone()),
					link: Some(
						format!(
							"https://twitter.com/{handle}/status/{id}",
							handle = self.handle,
							id = tweet.id
						)
						.as_str()
						.try_into()
						.expect("The URL is hand crafted and should always be valid"),
					),
					media: tweet.entities.media.as_ref().and_then(|x| {
						x.iter()
							.map(|x| match x.media_type {
								MediaType::Photo => {
									Some(Media::Photo(x.media_url.as_str().try_into().expect(
										"The tweet URL provided by the Tweeter API should always be a valid URL",
									)))
								}
								MediaType::Video => {
									Some(Media::Video(x.media_url.as_str().try_into().expect(
										"The tweet URL provided by the Tweeter API should always be a valid URL",
									)))
								}
								MediaType::Gif => None,
							})
							.collect::<Option<Vec<Media>>>()
					}),
					..Default::default()
				},
				..Default::default()
			})
			.collect::<Vec<_>>();

		Ok(messages)
	}
}

impl std::fmt::Debug for Twitter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Twitter")
			.field("handle", &self.handle)
			.finish_non_exhaustive()
	}
}
