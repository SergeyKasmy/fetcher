use std::time::Duration;

use crate::error::{NewsReaderError, Result};
use teloxide::{
	adaptors::{throttle::Limits, Throttle},
	payloads::SendMessageSetters,
	requests::{Request, Requester, RequesterExt},
	types::{ChatId, InputMedia, Message, ParseMode},
	Bot, RequestError,
};

pub struct Telegram {
	bot: Throttle<Bot>,
	chat_id: ChatId,
}

impl Telegram {
	pub fn new(bot: Bot, chat_id: impl Into<ChatId>) -> Self {
		Self {
			bot: bot.throttle(Limits::default()),
			chat_id: chat_id.into(),
		}
	}

	pub async fn send_text(&self, message: String) -> Result<Message> {
		loop {
			match self
				.bot
				.send_message(self.chat_id.clone(), &message)
				.parse_mode(ParseMode::Html)
				.disable_web_page_preview(true)
				.send()
				.await
			{
				Ok(message) => return Ok(message),
				Err(RequestError::RetryAfter(retry_after)) => {
					tokio::time::sleep(Duration::from_secs(retry_after as u64)).await;
				}
				Err(e) => return Err(NewsReaderError::Send { why: e.to_string() }),
			}
		}
	}

	pub async fn send_media(&self, media: Vec<InputMedia>) -> Result<Vec<Message>> {
		loop {
			match self
				.bot
				.send_media_group(self.chat_id.clone(), media.clone())
				.send()
				.await
			{
				Ok(messages) => return Ok(messages),
				Err(RequestError::RetryAfter(retry_after)) => {
					tokio::time::sleep(Duration::from_secs(retry_after as u64)).await;
				}
				Err(e) => return Err(NewsReaderError::Send { why: e.to_string() }),
			}
		}
	}
}
