/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
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

		// tried using Option here but ended up using unwrap_or_default() later anyways, so I'd thought why not use empty strings directly.
		// Sadly they aren't ZST but 24 bytes but that shouldn't be ~too~ much...
		let (mut head, tail) = match (&title, &link) {
			// if title and link are both presend
			(Some(title), Some(link)) => match self.link_location {
				// and the link should be in the title, then combine them
				LinkLocation::PreferTitle => {
					(format!("<a href=\"{link}\">{title}</a>\n"), String::new())
				}
				// even it should be at the bottom, return both separately
				LinkLocation::Bottom => (
					format!("{title}\n\n"),
					format!("\n<a href=\"{link}\">Link</a>"),
				),
			},
			// if only the title is presend, just print itself with an added newline
			(Some(title), None) => (format!("{title}\n\n"), String::new()),
			// and if only the link is present, but it at the bottom of the message, even if it should try to be in the title
			(None, Some(link)) => (String::new(), format!("\n<a href=\"{link}\">Link</a>")),
			(None, None) => (String::new(), String::new()),
		};

		if let Some(tag) = tag {
			head.insert_str(0, &format!("#{tag}\n\n"));
		}

		let text = {
			if body.chars().count() + head.chars().count() + tail.chars().count() > MAX_MSG_LEN {
				let mut msg_parts = split_into_multiple_msg(head, body, tail);
				let last = msg_parts.pop().unwrap(); // unwrap NOTE: we confirmed the entire message is too long, thus we should have at least one part

				for msg_part in msg_parts {
					self.send_text(msg_part).await?;
				}

				last
			} else {
				format!("{head}{body}{tail}")
			}
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

#[allow(clippy::needless_pass_by_value)] // I want to take ownership of the msg parts to avoid using them later by mistake
fn split_into_multiple_msg(head: String, body: String, tail: String) -> Vec<String> {
	let body_char_num = body.chars().count();
	let head_char_num = head.chars().count();
	let tail_char_num = tail.chars().count();

	assert!(head_char_num < MAX_MSG_LEN);
	assert!(tail_char_num < MAX_MSG_LEN);

	let mut parts: Vec<String> = Vec::new();

	// first part with head and as much body as we can fit
	let send_till = MAX_MSG_LEN - head.chars().count();
	let body_first_part = body.chars().take(send_till).collect::<String>();
	parts.push(format!("{head}{body_first_part}"));

	let mut next_body_part_from = send_till + 1;

	// split the rest of body into parts
	loop {
		if next_body_part_from >= body_char_num {
			break;
		}

		let next_body_part_chars_count =
			std::cmp::min(MAX_MSG_LEN, body_char_num - next_body_part_from);

		parts.push(
			body.chars()
				.skip(next_body_part_from)
				.take(MAX_MSG_LEN)
				.collect::<String>(),
		);
		next_body_part_from += next_body_part_chars_count;
	}

	// put tail into the last part of body if it fits, otherwise put it into it's owm part
	if let Some(last) = parts.last_mut() {
		if last.chars().count() < MAX_MSG_LEN - tail_char_num {
			last.push_str(&tail);
		} else {
			parts.push(tail);
		}
	} else {
		parts.push(tail);
	}

	parts
}

#[cfg(test)]
mod tests {
	use super::*;
	const MSG_COUNT: usize = 3;

	#[test]
	fn split_msg_empty_head_tail() {
		let head = String::new();

		let mut body = String::new();
		for _ in 0..MAX_MSG_LEN * MSG_COUNT {
			body.push('b');
		}

		let tail = String::new();

		let v = split_into_multiple_msg(head, body, tail);
		assert_eq!(v.len(), MSG_COUNT);
	}

	#[test]
	fn split_msg_long_head() {
		let mut head = String::new();
		for _ in 0..150 {
			head.push('h');
		}

		let mut body = String::new();
		for _ in 0..MAX_MSG_LEN * MSG_COUNT {
			body.push('b');
		}

		let tail = String::new();

		let v = split_into_multiple_msg(head, body, tail);
		assert_eq!(v.len(), MSG_COUNT + 1);
	}

	#[test]
	fn split_msg_with_tail_almost_fitting() {
		let head = String::new();

		let mut body = String::new();
		// body is 1 char from max msg len
		for _ in 0..MAX_MSG_LEN * MSG_COUNT - 1 {
			body.push('b');
		}

		let tail = "tt".to_owned(); // and tail is 2 char

		let v = split_into_multiple_msg(head, body, tail);
		assert_eq!(v.len(), MSG_COUNT + 1); // tail shouldn't be split and thus should be put into it's own msg
	}
}
