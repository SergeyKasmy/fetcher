use anyhow::Result;
use rss::Channel;
use std::time::Duration;
use teloxide::{
	adaptors::{throttle::Limits, Throttle},
	requests::{Request, Requester, RequesterExt},
	types::ChatId,
	Bot, RequestError,
};

pub struct NewsReader {
	name: String,
	rss: String,
	bot: Throttle<Bot>,
	chat_id: ChatId,
	http_client: reqwest::Client,
}

impl NewsReader {
	pub fn new(name: String, rss: String, bot: Bot, chat_id: impl Into<ChatId>) -> Self {
		Self {
			name,
			rss,
			bot: bot.throttle(Limits::default()),
			chat_id: chat_id.into(),
			http_client: reqwest::Client::new(),
		}
	}

	pub async fn start(&mut self) -> Result<()> {
		use std::fs;

		let last_read_guid = fs::read_to_string(format!("last_read_guid/{}.txt", self.name)).ok();
		let last_read_guid = self.send_news(last_read_guid).await?;
		if let Some(last_read_guid) = last_read_guid {
			let _ = fs::create_dir("last_read_guid");
			fs::write(format!("last_read_guid/{}.txt", self.name), last_read_guid)?;
		}

		Ok(())
	}

	async fn send_news(&mut self, mut last_read_guid: Option<String>) -> Result<Option<String>> {
		let mut feed = self.get_rss_feed().await?;

		if let Some(last_read_guid) = last_read_guid.as_deref() {
			if let Some(last_read_guid_pos) = feed
				.items
				.iter()
				.position(|x| x.guid.as_ref().unwrap().value == last_read_guid)
			{
				feed.items.drain(last_read_guid_pos..);
			}
		}
		for item in feed.items {
			loop {
				match self
					.bot
					.send_message(self.chat_id.clone(), item.description.as_ref().unwrap())
					.send()
					.await
				{
					Ok(message) => {
						eprintln!("Sent {:?}", message.text());
						break;
					}
					Err(RequestError::RetryAfter(retry_after)) => {
						eprintln!("Sleeping for {}s", retry_after);
						tokio::time::sleep(Duration::from_secs(retry_after as u64)).await;
						continue;
					}
					Err(e) => return Err(e.into()),
				}
			}
			last_read_guid = Some(item.guid.unwrap().value); // NOTE: crash if the feed item doesn't have a guid. That should never happen though
		}

		Ok(last_read_guid)
	}

	async fn get_rss_feed(&self) -> Result<Channel> {
		let content = self
			.http_client
			.get(&self.rss)
			.send()
			.await?
			.bytes()
			.await?;

		let channel = Channel::read_from(&content[..])?;
		Ok(channel)
	}
}
