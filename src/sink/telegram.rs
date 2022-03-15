/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::time::Duration;
use teloxide::{
	// adaptors::{throttle::Limits, Throttle},
	payloads::SendMessageSetters,
	requests::{Request, Requester},
	types::{
		ChatId, InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, Message as TelMessage,
		ParseMode,
	},
	ApiError,
	Bot,
	RequestError,
};

use crate::{
	error::{Error, Result},
	sink::{message::LinkLocation, Media, Message},
};

pub struct Telegram {
	// bot: Throttle<Bot>,
	bot: Bot,
	chat_id: ChatId,
}

/// Make the message text more logging friendly:
/// 1. Remove the opening html tag if it begins with one
/// 2. Shorten the message to 150 chars
fn fmt_comment_msg_text(s: &str) -> String {
	let s = if s.starts_with('<') {
		if let Some(tag_end) = s.find('>') {
			&s[tag_end..]
		} else {
			s
		}
	} else {
		s
	};

	s.chars().take(/* shorten to */ 40 /* chars */).collect()
}

impl Telegram {
	#[must_use]
	pub fn new(bot: Bot, chat_id: impl Into<ChatId>) -> Self {
		Self {
			// TODO: THIS BLOCKS. WHY??????
			// #2 throttle() spawns a tokio task but we are in sync. Maybe that causes the hangup?
			// bot: bot.throttle(Limits::default()),
			bot,
			chat_id: chat_id.into(),
		}
	}

	#[allow(clippy::items_after_statements)] // TODO
	#[tracing::instrument(skip_all,
	fields(
		body = fmt_comment_msg_text(&message.body).as_str(),
		)
	)]
	pub async fn send(&self, message: Message) -> Result<()> {
		let Message {
			title,
			body,
			link,
			media,
		} = message;

		// TODO: replace upticks ` with teloxide::utils::html::escape_code

		// NOTE: emails/html sites often contain all kinds of html or other text which Telegram's HTML parser doesn't approve of
		// I dislike the need to add an extra dependency just for this simple task but you gotta do what you gotta do.
		// Hopefully I'll find a better way to escape everything though since I don't fear a possibility that it'll be
		// somehow harmful 'cause it doesn't consern me, only Telegram :P
		let body = ammonia::clean(&body);

		tracing::debug!(
			"Processing message: len: {}, media: {}",
			body.len(),
			media.is_some()
		);

		const PADDING: usize = 10; // how much free space to reserve for new lines and "Link" buttons. 10 should be enough
		let title_len = title.as_ref().map_or(0, String::len);
		let approx_msg_len = title_len + body.len() + PADDING;
		let body = if title_len + body.len() + PADDING > 4096 {
			// TODO: split the message properly instead of just throwing the rest away
			tracing::warn!("Message too long ({approx_msg_len})");
			let (idx, _) = body.char_indices().nth(4096 - title_len - PADDING).unwrap(); // unwrap FIXME: len is measured in bytes while chat_indices is in graphemes. That may cause crashes.
																			 // It would be just better to split the message in parts and send that instead of trying to fix this
			let mut m = body[..idx].to_string();
			m.push_str("...");
			tracing::debug!("Message body length after trimming: {}", m.len());
			tracing::debug!("Message full approx len: {}", m.len() + title_len + PADDING);
			m
		} else {
			body
		};

		let text = match (&title, &link) {
			(Some(title), Some(link)) => match link.loc {
				LinkLocation::PreferTitle => {
					format!("<a href=\"{url}\">{title}</a>\n{body}", url = link.url,)
				}
				LinkLocation::Bottom => {
					format!(
						"{title}\n{body}\n<a href=\"{url}\">Link</a>",
						url = link.url
					)
				}
			},
			(Some(title), None) => {
				format!("{title}\n{body}")
			}
			(None, Some(link)) => {
				format!("{body}\n<a href=\"{url}\">Link</a>", url = link.url)
			}
			(None, None) => body,
		};

		if let Some(media) = media {
			match self
				.send_media(
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
				.await
			{
				Err(Error::Telegram(RequestError::Api(ApiError::Unknown(e)), _))
					if e == "Bad Request: wrong file identifier/HTTP URL specified" =>
				{
					// TODO: reupload the image manually if this happens
					tracing::warn!("Telegram disapproved of the media URL ({e}), sending the message as pure text");
					self.send_text(text).await?;
				}
				Ok(_) => (),
				Err(e) => return Err(e),
			}
		} else {
			self.send_text(text).await?;
		}

		Ok(())
	}

	// TODO: move error handling out to dedup send_text & send_media
	async fn send_text(&self, message: String) -> Result<TelMessage> {
		loop {
			tracing::info!("Sending text message");
			tracing::trace!("Message contents: {message:?}");

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
					tracing::warn!("Exceeded rate limit, retrying in {retry_after}s");
					tokio::time::sleep(Duration::from_secs(retry_after.try_into().expect(
						"Telegram returned a negative RetryAfter value. How did that even happen?",
					)))
					.await;
				}
				Err(e) => {
					return Err((
						e,
						Box::new(message) as Box<dyn std::fmt::Debug + Send + Sync>,
					)
						.into())
				}
			}
		}
	}

	async fn send_media(&self, media: Vec<InputMedia>) -> Result<Vec<TelMessage>> {
		loop {
			tracing::info!("Sending media message");
			tracing::trace!("Message contents: {media:?}");

			match self
				.bot
				.send_media_group(self.chat_id.clone(), media.clone())
				.send()
				.await
			{
				Ok(messages) => return Ok(messages),
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::warn!("Exceeded rate limit, retrying in {retry_after}s");
					tokio::time::sleep(Duration::from_secs(retry_after.try_into().expect(
						"Telegram returned a negative RetryAfter value. How did that even happen?",
					)))
					.await;
				}
				Err(RequestError::Api(ApiError::Unknown(err_str)))
					if err_str == "Bad Request: failed to get HTTP URL content" =>
				{
					tracing::warn!("{err_str}. Retrying in 30 seconds");
					tokio::time::sleep(Duration::from_secs(30)).await;
				}
				Err(e) => {
					return Err(
						(e, Box::new(media) as Box<dyn std::fmt::Debug + Send + Sync>).into(),
					)
				}
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

// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use std::env::var;

// 	#[tokio::test]
// 	async fn send_text_too_long() {
// 		let tg = Telegram::new(
// 			Bot::new(var("BOT_TOKEN").unwrap()),
// 			var("DEBUG_CHAT_ID").unwrap(),
// 		);
// 		let mut long_text = String::with_capacity(8392);

// 		for _ in 0..4096 {
// 			long_text.push('0');
// 		}

// 		for _ in 0..4096 {
// 			long_text.push('1');
// 		}

// 		for _ in 0..200 {
// 			long_text.push('2');
// 		}

// 		// tg.send_text(too_long_text).await.unwrap();
// 		tg.send(Message {
// 			text: long_text,
// 			media: None,
// 		})
// 		.await
// 		.unwrap();
// 	}
// }
