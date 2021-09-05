use crate::guid::{get_last_read_guid, save_last_read_guid};

use anyhow::Result;
use egg_mode::{auth::bearer_token, tweet::user_timeline, user::UserID, KeyPair, Token};
use teloxide::{
	requests::{Request, Requester},
	types::ChatId,
	Bot,
};

pub struct TwitterNewsReader {
	name: &'static str,
	handle: UserID,
	token: Token,
	filters: Option<&'static [&'static str]>,
	bot: Bot,
	chat_id: ChatId,
}

impl TwitterNewsReader {
	pub async fn new(
		name: &'static str,
		handle: &'static str,
		api_key: String,
		api_key_secret: String,
		filters: Option<&'static [&'static str]>,
		bot: Bot,
		chat_id: impl Into<ChatId>,
	) -> Result<Self> {
		Ok(Self {
			name,
			handle: handle.into(),
			token: bearer_token(&KeyPair::new(api_key, api_key_secret)).await?,
			filters,
			bot,
			chat_id: chat_id.into(),
		})
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

	async fn send_news(&mut self, mut last_read_guid: Option<u64>) -> Result<Option<u64>> {
		eprintln!("Last Read GUID: {:?}", last_read_guid);
		let (_, tweets) = user_timeline(self.handle.clone(), false, true, &self.token)
			.older(last_read_guid)
			.await?;
		for tweet in tweets.iter().rev() {
			if let Some(filters) = self.filters {
				if !Self::tweet_contains_filters(&tweet.text, filters) {
					continue;
				}
			}
			let message = self
				.bot
				.send_message(self.chat_id.clone(), &tweet.text)
				.send()
				.await?;
			eprintln!("Sent {:?}", message.text());

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
