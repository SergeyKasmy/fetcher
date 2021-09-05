use crate::error::NewsReaderError;
use crate::error::Result;
use crate::guid::{get_last_read_guid, save_last_read_guid};

use rss::Channel;
use std::time::Duration;
use teloxide::{
	adaptors::{throttle::Limits, Throttle},
	payloads::SendMessageSetters,
	requests::{Request, Requester, RequesterExt},
	types::{ChatId, ParseMode},
	Bot, RequestError,
};

pub struct RssNewsReader {
	name: &'static str,
	rss: &'static str,
	bot: Throttle<Bot>,
	chat_id: ChatId,
	http_client: reqwest::Client,
}

impl RssNewsReader {
	pub fn new(
		name: &'static str,
		rss: &'static str,
		bot: Bot,
		chat_id: impl Into<ChatId>,
	) -> Self {
		Self {
			name,
			rss,
			bot: bot.throttle(Limits::default()),
			chat_id: chat_id.into(),
			http_client: reqwest::Client::new(),
		}
	}

	pub async fn start(&mut self) -> Result<()> {
		let last_read_guid = self.send_news(get_last_read_guid(self.name)).await?;
		if let Some(last_read_guid) = last_read_guid {
			save_last_read_guid(self.name, last_read_guid)?;
		}

		Ok(())
	}

	async fn send_news(&mut self, mut last_read_guid: Option<String>) -> Result<Option<String>> {
		let content = self
			.http_client
			.get(self.rss)
			.send()
			.await
			.map_err(|e| NewsReaderError::RssGet {
				feed: self.name,
				why: e.to_string(),
			})?
			.bytes()
			.await
			.map_err(|e| NewsReaderError::RssGet {
				feed: self.name,
				why: e.to_string(),
			})?;
		let mut feed = Channel::read_from(&content[..]).map_err(|e| NewsReaderError::RssParse {
			feed: self.name,
			why: e.to_string(),
		})?;

		if let Some(last_read_guid) = last_read_guid.as_deref() {
			if let Some(last_read_guid_pos) = feed
				.items
				.iter()
				.position(|x| x.guid.as_ref().unwrap().value == last_read_guid)
			{
				feed.items.drain(last_read_guid_pos..);
			}
		}
		for item in feed.items.into_iter().rev() {
			let message = format!(
				"<a href=\"{}\">{}</a>\n{}",
				item.link.unwrap(),
				item.title.unwrap(),
				item.description.unwrap()
			); // NOTE: these fields are requred
			loop {
				match self
					.bot
					.send_message(self.chat_id.clone(), &message)
					.parse_mode(ParseMode::Html)
					.disable_web_page_preview(true)
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
					Err(e) => return Err(NewsReaderError::Telegram(e.to_string())),
				}
			}
			last_read_guid = Some(item.guid.unwrap().value); // NOTE: crash if the feed item doesn't have a guid. That should never happen though
		}

		Ok(last_read_guid)
	}
}
