use crate::error::Error;
use crate::error::Result;
use crate::guid::{get_last_read_guid, save_last_read_guid};
use crate::telegram::Telegram;

use egg_mode::entities::MediaType;
use egg_mode::{auth::bearer_token, tweet::user_timeline, KeyPair, Token};
use teloxide::types::{ChatId, InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, ParseMode};
use teloxide::Bot;

pub struct Twitter {
	name: &'static str,
	pretty_name: &'static str,
	handle: &'static str,
	token: Token,
	filters: Option<&'static [&'static str]>,
	telegram: Telegram,
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
		bot: Bot,
		chat_id: impl Into<ChatId>,
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
			telegram: Telegram::new(bot, chat_id),
		})
	}

	async fn send_news(&mut self, mut last_read_guid: Option<u64>) -> Result<Option<u64>> {
		let (_, tweets) = user_timeline(self.handle, false, true, &self.token)
			.older(last_read_guid)
			.await
			.map_err(|e| Error::Get {
				service: "Twitter".to_string(),
				why: e.to_string(),
			})?;
		for tweet in tweets.iter().rev() {
			if let Some(filters) = self.filters {
				if !Self::tweet_contains_filters(&tweet.text, filters) {
					continue;
				}
			}
			let message = format!(
				"#{}\n\n{}\n<a href=\"https://twitter.com/{}/status/{}\">Link</a>",
				self.pretty_name, tweet.text, self.handle, tweet.id
			);
			if let Some(twitter_media) = &tweet.entities.media {
				let tg_media = twitter_media
					.iter()
					.filter_map(|x| match x.media_type {
						MediaType::Photo => Some(InputMedia::Photo(
							InputMediaPhoto::new(InputFile::url(x.media_url.clone()))
								.caption(message.clone())
								.parse_mode(ParseMode::Html),
						)),
						MediaType::Video => Some(InputMedia::Video(
							InputMediaVideo::new(InputFile::url(x.media_url.clone()))
								.caption(message.clone())
								.parse_mode(ParseMode::Html),
						)),
						_ => None,
					})
					.collect();
				self.telegram.send_media(tg_media).await?;
			} else {
				self.telegram.send_text(message).await?;
			}

			last_read_guid = Some(tweet.id);
		}

		Ok(last_read_guid)
	}

	fn tweet_contains_filters(tweet: &str, filters: &[&str]) -> bool {
		for filter in filters {
			if !tweet.to_lowercase().contains(&filter.to_lowercase()) {
				return false;
			}
		}

		true
	}
	pub async fn start(&mut self) -> Result<()> {
		let last_read_guid = self
			.send_news(get_last_read_guid(self.name).and_then(|x| x.trim().parse::<u64>().ok()))
			.await?;
		if let Some(last_read_guid) = last_read_guid {
			save_last_read_guid(self.name, last_read_guid.to_string())?;
		}

		Ok(())
	}
}
