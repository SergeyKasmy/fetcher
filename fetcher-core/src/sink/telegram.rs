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
	error::sink::Error as SinkError,
	sink::{Media, Message},
};

const MAX_MSG_LEN: usize = 4096;

/// Either embed the link into the title or put it as a separate "Link" button at the botton of the message.
/// `PreferTitle` falls back to `Bottom` if Message.title is None
#[derive(Clone, Copy, Default, Debug)]
pub enum LinkLocation {
	PreferTitle,
	#[default]
	Bottom,
}

pub struct Telegram {
	// bot: Throttle<Bot>,
	bot: Bot,
	chat_id: ChatId,
	link_location: LinkLocation,
}

impl Telegram {
	#[must_use]
	pub fn new(bot: Bot, chat_id: i64, link_location: LinkLocation) -> Self {
		Self {
			// TODO: THIS BLOCKS. WHY??????
			// #2 throttle() spawns a tokio task but we are in sync. Maybe that causes the hangup?
			// bot: bot.throttle(Limits::default()),
			bot,
			chat_id: ChatId(chat_id),
			link_location,
		}
	}

	#[allow(clippy::items_after_statements)] // TODO
	#[tracing::instrument(skip_all)]
	pub async fn send(&self, message: Message, tag: Option<&str>) -> Result<(), SinkError> {
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
			"Processing message: title: {title:?}, body len: {blen}, media: {m}",
			blen = body.len(),
			m = media.is_some(),
		);

		let text = {
			let mut text = match (&title, &link) {
				(Some(title), Some(link)) => match self.link_location {
					LinkLocation::PreferTitle => {
						format!("<a href=\"{link}\">{title}</a>\n{body}")
					}
					LinkLocation::Bottom => {
						format!("{title}\n{body}\n<a href=\"{link}\">Link</a>",)
					}
				},
				(Some(title), None) => {
					format!("{title}\n{body}")
				}
				(None, Some(link)) => {
					format!("{body}\n<a href=\"{link}\">Link</a>")
				}
				(None, None) => body,
			};

			if let Some(tag) = tag {
				text.insert_str(0, &format!("#{tag}\n\n"));
			}

			text
		};

		// FIXME: bug: if the message is just a couple of bytes over the limit and ends with a link, it could split in in two, e.g. 1) <a hre 2) f="">Link</a> and thus break it
		let text = {
			if text.len() > MAX_MSG_LEN {
				let mut parts = Vec::new();
				let mut begin_slice_from = 0;

				loop {
					if begin_slice_from == text.len() {
						break;
					}

					let till = std::cmp::min(begin_slice_from + MAX_MSG_LEN, text.len());

					// this assumes that the string slice is valid which it may not be
					#[allow(clippy::string_from_utf8_as_bytes)]
					match std::str::from_utf8(&text.as_bytes()[begin_slice_from..till]) {
						Ok(s) => {
							parts.push(s);
							begin_slice_from = till;
						}
						Err(e) => {
							let valid_up_to = e.valid_up_to();
							let s = &text[begin_slice_from..valid_up_to];
							parts.push(s);
							begin_slice_from = valid_up_to;
						}
					}
				}

				let last_part = parts.pop().unwrap();
				for part in parts {
					self.send_text(part.to_owned()).await?;
				}

				last_part.to_owned()
			} else {
				text
			}
		};

		// assert!(text.len() < MAX_MSG_LEN);

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
				Err(SinkError::Telegram {
					source: RequestError::Api(ApiError::Unknown(e)),
					msg: _,
				}) if e.contains("Failed to get HTTP URL content")
					|| e.contains("Wrong file identifier/HTTP URL specified") =>
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
	async fn send_text(&self, message: String) -> Result<TelMessage, SinkError> {
		loop {
			tracing::info!("Sending text message");
			// TODO: move to the Sink::send() despetcher method mb
			tracing::trace!("Message contents: {message:?}");

			match self
				.bot
				.send_message(self.chat_id, &message)
				.parse_mode(ParseMode::Html)
				.disable_web_page_preview(true)
				.send()
				.await
			{
				Ok(message) => return Ok(message),
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::warn!(
						"Exceeded rate limit, retrying in {}s",
						retry_after.as_secs()
					);
					tokio::time::sleep(retry_after).await;
				}
				Err(e) => {
					return Err(SinkError::Telegram {
						source: e,
						msg: Box::new(message),
					});
				}
			}
		}
	}

	async fn send_media(&self, media: Vec<InputMedia>) -> Result<Vec<TelMessage>, SinkError> {
		loop {
			tracing::info!("Sending media message");
			tracing::trace!("Message contents: {media:?}");

			match self
				.bot
				.send_media_group(self.chat_id, media.clone())
				.send()
				.await
			{
				Ok(messages) => return Ok(messages),
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::warn!(
						"Exceeded rate limit, retrying in {}s",
						retry_after.as_secs()
					);
					tokio::time::sleep(retry_after).await;
				}
				Err(RequestError::Api(ApiError::Unknown(err_str)))
					if err_str == "Bad Request: failed to get HTTP URL content" =>
				{
					tracing::warn!("{err_str}. Retrying in 30 seconds");
					tokio::time::sleep(Duration::from_secs(30)).await;
				}
				Err(e) => {
					return Err(SinkError::Telegram {
						source: e,
						msg: Box::new(media),
					});
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
// 		eprintln!("Running");
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

// 		eprintln!("Message constructed");
// 		tg.send(
// 			Message {
// 				// title: Some("Testing title".to_owned()),
// 				title: None,
// 				body: long_text,
// 				link: None,
// 				media: None,
// 			},
// 			None,
// 		)
// 		.await
// 		.unwrap();
// 		eprintln!("Message sent");
// 	}
// }
