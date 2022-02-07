/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::time::Duration;
use teloxide::{
	adaptors::{throttle::Limits, Throttle},
	payloads::SendMessageSetters,
	requests::{Request, Requester, RequesterExt},
	types::{
		ChatId, InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, Message as TelMessage,
		ParseMode,
	},
	Bot, RequestError,
};

use crate::error::Result;
use crate::sink::Media;
use crate::sink::Message;

pub struct Telegram {
	bot: Throttle<Bot>,
	chat_id: ChatId,
}

/// Make the message text more logging friendly:
/// 1. Remove the opening html tag if it begins with one
/// 2. Shorten the message to 150 chars
fn fmt_comment_msg(s: &str) -> String {
	let s = if s.starts_with('<') {
		if let Some(tag_end) = s.find('>') {
			&s[tag_end..]
		} else {
			s
		}
	} else {
		s
	};

	s.chars().take(/* shorten to */ 150 /* chars */).collect()
}

impl Telegram {
	pub fn new(bot: Bot, chat_id: impl Into<ChatId>) -> Self {
		Self {
			bot: bot.throttle(Limits::default()),
			chat_id: chat_id.into(),
		}
	}

	#[tracing::instrument(skip(message), fields(len = message.text.len(), text = fmt_comment_msg(&message.text).as_str(), media.is_some = message.media.is_some()))]
	pub async fn send(&self, message: Message) -> Result<()> {
		// workaround for some kind of a bug that doesn't let access both text and media fields of the struct in the map closure at once
		let text = if message.text.len() > 4096 {
			// TODO: split the message properly instead of just throwing the rest away
			tracing::warn!("Message too long ({len})", len = message.text.len());
			let (idx, _) = message.text.char_indices().nth(4096 - 3).unwrap(); // unwrap NOTE: safe, length already confirmed to be bigger
			let mut m = message.text[..idx].to_string();
			m.push_str("...");
			m
		} else {
			message.text
		};

		if let Some(media) = message.media {
			self.send_media(
				media
					.into_iter()
					.map(|x| match x {
						Media::Photo(url) => InputMedia::Photo(
							InputMediaPhoto::new(InputFile::url(url))
								.caption(text.clone())
								.parse_mode(ParseMode::Html),
						),
						Media::Video(url) => InputMedia::Video(
							InputMediaVideo::new(InputFile::url(url))
								.caption(text.clone())
								.parse_mode(ParseMode::Html),
						),
					})
					.collect::<Vec<InputMedia>>(),
			)
			.await?;
		} else {
			self.send_text(text).await?;
		}

		Ok(())
	}

	// TODO: move error handling out to dedup send_text & send_media
	async fn send_text(&self, message: String) -> Result<TelMessage> {
		loop {
			tracing::info!("Sending text message");
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
					tracing::warn!("Exceeded rate limit, retrying in {retry_after}");
					tokio::time::sleep(Duration::from_secs(retry_after as u64)).await;
				}
				Err(e) => return Err(e.into()),
			}
		}
	}

	async fn send_media(&self, media: Vec<InputMedia>) -> Result<Vec<TelMessage>> {
		loop {
			tracing::info!("Sending media message");
			match self
				.bot
				.send_media_group(self.chat_id.clone(), media.clone())
				.send()
				.await
			{
				Ok(messages) => return Ok(messages),
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::warn!("Exceeded rate limit, retrying in {retry_after}");
					tokio::time::sleep(Duration::from_secs(retry_after as u64)).await;
				}
				Err(e) => return Err(e.into()),
			}
		}
	}
}

impl std::fmt::Debug for Telegram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Telegram")
			.field("chat_id", &self.chat_id)
			.finish_non_exhaustive()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::env::var;

	#[tokio::test]
	async fn send_text_too_long() {
		let tg = Telegram::new(
			Bot::new(var("BOT_TOKEN").unwrap()),
			var("DEBUG_CHAT_ID").unwrap(),
		);
		let mut long_text = String::with_capacity(8392);

		for _ in 0..4096 {
			long_text.push('0');
		}

		for _ in 0..4096 {
			long_text.push('1');
		}

		for _ in 0..200 {
			long_text.push('2');
		}

		// tg.send_text(too_long_text).await.unwrap();
		tg.send(Message {
			text: long_text,
			media: None,
		})
		.await
		.unwrap();
	}
}
