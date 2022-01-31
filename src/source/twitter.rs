use crate::error::Error;
use crate::error::Result;
use crate::sink::Media;
use crate::sink::Message;
use crate::settings::{last_read_id, save_last_read_id};

use egg_mode::entities::MediaType;
use egg_mode::{auth::bearer_token, tweet::user_timeline, KeyPair, Token};

#[derive(Debug)]
pub struct Twitter {
	name: String,
	pretty_name: String,
	handle: String,
	token: Token,
	filter: Vec<String>,
}

impl Twitter {
	#[allow(clippy::too_many_arguments)]
	#[tracing::instrument]
	pub async fn new(
		name: String,
		pretty_name: String,
		handle: String,
		api_key: String,
		api_key_secret: String,
		filter: Vec<String>,
	) -> Result<Self> {
		tracing::info!("Creatng a Twitter provider");
		Ok(Self {
			name,
			pretty_name,
			handle,
			token: bearer_token(&KeyPair::new(api_key, api_key_secret))
				.await
				.map_err(|e| Error::SourceAuth {
					service: "Twitter".to_string(),
					why: e.to_string(),
				})?,
			filter,
		})
	}

	#[tracing::instrument]
	pub async fn get(&mut self) -> Result<Vec<Message>> {
		let mut last_read_id = last_read_id(&self.name)?;
		let (_, tweets) = user_timeline(self.handle.clone(), false, true, &self.token) // FIXME: remove clone
			.older(last_read_id.as_ref().and_then(|x| x.parse().ok()))
			.await
			.map_err(|e| Error::SourceFetch {
				service: "Twitter".to_string(),
				why: e.to_string(),
			})?;
		tracing::debug!("Got {amount} tweets", amount = tweets.len());

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

				last_read_id = Some(tweet.id.to_string());
				Some(Message {
					text,
					media: tweet.entities.media.as_ref().and_then(|x| {
						x.iter()
							.map(|x| match x.media_type {
								MediaType::Photo => Some(Media::Photo(x.media_url.clone())),
								MediaType::Video => Some(Media::Video(x.media_url.clone())),
								MediaType::Gif => None,
							})
							.collect::<Option<Vec<Media>>>()
					}),
				})
			})
			.collect::<Vec<Message>>();

		if let Some(id) = last_read_id {
			save_last_read_id(&self.name, id)?;
		}
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
