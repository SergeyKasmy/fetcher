use crate::error::NewsReaderError;
use crate::error::Result;
use crate::guid::{get_last_read_guid, save_last_read_guid};
use crate::telegram::Telegram;

use rss::Channel;
use teloxide::types::ChatId;
use teloxide::Bot;

pub struct RssNewsReader {
	name: &'static str,
	rss: &'static str,
	telegram: Telegram,
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
			telegram: Telegram::new(bot, chat_id),
			http_client: reqwest::Client::new(),
		}
	}

	pub async fn start(&mut self) -> Result<()> {
		if let Some(last_read_guid) = self.send_news(get_last_read_guid(self.name)).await? {
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
			.map_err(|e| NewsReaderError::Get {
				service: format!("RSS: {}", self.name),
				why: e.to_string(),
			})?
			.bytes()
			.await
			.map_err(|e| NewsReaderError::Get {
				service: format!("RSS: {}", self.name),
				why: e.to_string(),
			})?;
		let mut feed = Channel::read_from(&content[..]).map_err(|e| NewsReaderError::Parse {
			service: format!("RSS: {}", self.name),
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
			self.telegram.send_text(message).await?;
			last_read_guid = Some(item.guid.unwrap().value); // NOTE: crash if the feed item doesn't have a guid. That should never happen though
		}

		Ok(last_read_guid)
	}
}
