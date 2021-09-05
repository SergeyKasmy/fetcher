use crate::error::NewsReaderError;
use crate::guid::{get_last_read_guid, save_last_read_guid};

use egg_mode::entities::MediaType;
use egg_mode::{auth::bearer_token, tweet::user_timeline, KeyPair, Token};
use teloxide::types::{InputFile, InputMedia, InputMediaPhoto, InputMediaVideo};
use teloxide::{
	payloads::SendMessageSetters,
	requests::{Request, Requester},
	types::{ChatId, ParseMode},
	Bot,
};

pub struct TwitterNewsReader {
	name: &'static str,
	pretty_name: &'static str,
	handle: &'static str,
	token: Token,
	filters: Option<&'static [&'static str]>,
	bot: Bot,
	chat_id: ChatId,
}

impl TwitterNewsReader {
	pub async fn new(
		name: &'static str,
		pretty_name: &'static str,
		handle: &'static str,
		api_key: String,
		api_key_secret: String,
		filters: Option<&'static [&'static str]>,
		bot: Bot,
		chat_id: impl Into<ChatId>,
	) -> Result<Self, NewsReaderError> {
		Ok(Self {
			name,
			pretty_name,
			handle,
			token: bearer_token(&KeyPair::new(api_key, api_key_secret))
				.await
				.map_err(|e| NewsReaderError::Auth {
					service: "Twitter",
					why: e.to_string(),
				})?,
			filters,
			bot,
			chat_id: chat_id.into(),
		})
	}

	pub async fn start(&mut self) -> Result<(), NewsReaderError> {
		let last_read_guid = self
			.send_news(get_last_read_guid(self.name).and_then(|x| x.trim().parse::<u64>().ok()))
			.await?;
		if let Some(last_read_guid) = last_read_guid {
			save_last_read_guid(self.name, last_read_guid.to_string())?;
		}

		Ok(())
	}

	async fn send_news(
		&mut self,
		mut last_read_guid: Option<u64>,
	) -> Result<Option<u64>, NewsReaderError> {
		let (_, tweets) = user_timeline(self.handle.clone(), false, true, &self.token)
			.older(last_read_guid)
			.await
			.map_err(|e| NewsReaderError::Twitter {
				handle: self.handle,
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
					.into_iter()
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
					});
				self.bot
					.send_media_group(self.chat_id.clone(), tg_media)
					.send()
					.await
					.map_err(|e| NewsReaderError::Telegram(e.to_string()))?;
			} else {
				self.bot
					.send_message(self.chat_id.clone(), &message)
					.parse_mode(ParseMode::Html)
					.disable_web_page_preview(true)
					.send()
					.await
					.map_err(|e| NewsReaderError::Telegram(e.to_string()))?;
				eprintln!("Sent {:?}", message);
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
}
