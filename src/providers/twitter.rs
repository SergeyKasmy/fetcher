use crate::error::Error;
use crate::error::Result;
use crate::guid::Guid;
use crate::telegram::Media;
use crate::telegram::Message;

use egg_mode::entities::MediaType;
use egg_mode::{auth::bearer_token, tweet::user_timeline, KeyPair, Token};

#[derive(Debug)]
pub struct Twitter {
	name: &'static str,
	pretty_name: &'static str,
	handle: &'static str,
	token: Token,
	filters: Option<&'static [&'static str]>,
}

impl Twitter {
	#[allow(clippy::too_many_arguments)]
	pub async fn new(
		name: &'static str,
		pretty_name: &'static str,
		handle: &'static str,
		api_key: String,
		api_key_secret: String,
		filters: Option<&'static [&'static str]>,
	) -> Result<Self> {
		Ok(Self {
			name,
			pretty_name,
			handle,
			token: bearer_token(&KeyPair::new(api_key, api_key_secret))
				.await
				.map_err(|e| Error::Auth {
					service: "Twitter".to_string(),
					why: e.to_string(),
				})?,
			filters,
		})
	}

	pub async fn get(&mut self) -> Result<Vec<Message>> {
		let mut last_read_guid = Guid::new(self.name)?;
		let (_, tweets) = user_timeline(self.handle, false, true, &self.token)
			.older(last_read_guid.guid.parse().ok())
			.await
			.map_err(|e| Error::Get {
				service: "Twitter".to_string(),
				why: e.to_string(),
			})?;

		let messages = tweets
			.iter()
			.rev()
			.filter_map(|tweet| {
				if let Some(filters) = self.filters {
					if !Self::tweet_contains_filters(&tweet.text, filters) {
						return None;
					}
				}

				let text = format!(
					"#{}\n\n{}\n<a href=\"https://twitter.com/{}/status/{}\">Link</a>",
					self.pretty_name, tweet.text, self.handle, tweet.id
				);

				last_read_guid.guid = tweet.id.to_string();
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

		last_read_guid.save()?;

		Ok(messages)
	}

	fn tweet_contains_filters(tweet: &str, filters: &[&str]) -> bool {
		for filter in filters {
			if !tweet.to_lowercase().contains(&filter.to_lowercase()) {
				return false;
			}
		}

		true
	}
}
