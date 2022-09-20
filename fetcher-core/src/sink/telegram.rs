/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Telegram`] sink

use crate::{
	error::sink::Error as SinkError,
	sink::{Media, Message},
};

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
use url::Url;

// FIXME: it's 1024 for media captions and 4096 for normal messages
const MAX_MSG_LEN: usize = 1024;

/// Telegram sink. Supports text and media messages and embeds text into media captions if present. Automatically splits the text into separate messages if it's too long
pub struct Telegram {
	bot: Bot,
	chat_id: ChatId,
	link_location: LinkLocation,
}

/// Where to put `message.link`
#[derive(Clone, Copy, Default, Debug)]
pub enum LinkLocation {
	/// Try to put in the title but fall back to `Bottom` if `Message.link` is None
	PreferTitle,
	/// Put the link at the bottom of the message in a "Link" button
	#[default]
	Bottom,
}

impl Telegram {
	/// Creates a new Telegram sink using the bot `token` that sends messages to chat with `chat_id` with `Message.link` put at `link_location`
	#[must_use]
	pub fn new(token: String, chat_id: i64, link_location: LinkLocation) -> Self {
		Self {
			// TODO: THIS BLOCKS. WHY??????
			// #2 throttle() spawns a tokio task but we are in sync. Maybe that causes the hangup?
			// bot: bot.throttle(Limits::default()),
			bot: Bot::new(token),
			chat_id: ChatId(chat_id),
			link_location,
		}
	}

	/// Sends a `message` with `tag`, if specified
	#[tracing::instrument(skip_all)]
	pub async fn send(&self, message: Message, tag: Option<&str>) -> Result<(), SinkError> {
		let Message {
			title,
			body,
			link,
			media,
		} = message;

		tracing::debug!(
			"Processing message: title: {title:?}, body len: {}, link: {}, media: {}",
			body.as_ref().map_or(0, String::len),
			link.is_some(),
			media.is_some(),
		);

		let title = title.map(|s| teloxide::utils::html::escape(&s));
		let body = body.map(|s| teloxide::utils::html::escape(&s));

		let (head, tail) = self.format_head_tail(title, link, tag);
		let body = body.unwrap_or_default();

		// TODO: send media with the first message
		// TODO: maybe add an option to make all consecutive messages reply to the prev ones
		let text = {
			if body.chars().count() + head.chars().count() + tail.chars().count() > MAX_MSG_LEN {
				let mut msg_parts = split_msg_into_parts(head, body, tail);
				let last = msg_parts.pop().expect("The entire message is confirmed to be too long and thus the split fn should always return at least 2 message parts");

				for msg_part in msg_parts {
					self.send_text(&msg_part).await?;
				}

				last
			} else {
				format!("{head}{body}{tail}")
			}
		};

		if let Some(media) = media {
			let media = media
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
				.collect::<Vec<InputMedia>>();

			match self.send_media(media).await {
				Err(SinkError::Telegram {
					source: RequestError::Api(ApiError::Unknown(e)),
					msg: _,
				}) if e.contains("Failed to get HTTP URL content")
					|| e.contains("Wrong file identifier/HTTP URL specified") =>
				{
					// TODO: reupload the image manually if this happens
					tracing::warn!("Telegram disapproved of the media URL ({e}), sending the message as pure text");
					self.send_text(&text).await?;
				}
				Ok(_) => (),
				Err(e) => return Err(e),
			}
		} else {
			self.send_text(&text).await?;
		}

		Ok(())
	}

	// TODO: move error handling out to dedup send_text & send_media
	async fn send_text(&self, message: &str) -> Result<TelMessage, SinkError> {
		loop {
			tracing::info!("Sending text message");
			// TODO: move to the Sink::send() despetcher method mb
			tracing::trace!("Message contents: {message:?}");

			match self
				.bot
				.send_message(self.chat_id, message)
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
						msg: Box::new(message.to_owned()),
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

	fn format_head_tail(
		&self,
		title: Option<String>,
		link: Option<Url>,
		tag: Option<&str>,
	) -> (String, String) {
		// tried using Option here but ended up using unwrap_or_default() later anyways, so I'd thought why not use empty strings directly.
		// Sadly they aren't ZST but 24 bytes but that shouldn't be ~too~ much...
		let (mut head, tail) = match (title, link) {
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
			let tag = tag.replace(
				|c| match c {
					'_' => false,
					c if c.is_alphabetic() || c.is_ascii_digit() => false,
					_ => true,
				},
				"_",
			);

			head.insert_str(0, &format!("#{tag}\n\n",));
		}

		(head, tail)
	}
}

#[allow(clippy::needless_pass_by_value)] // I want to take ownership of the msg parts to avoid using them later by mistake
fn split_msg_into_parts(head: String, body: String, tail: String) -> Vec<String> {
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

		let part = body
			.chars()
			.skip(next_body_part_from)
			.take(MAX_MSG_LEN)
			.collect::<String>();

		if !part.is_empty() {
			parts.push(part);
		}

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

impl std::fmt::Debug for Telegram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Telegram")
			.field("chat_id", &self.chat_id)
			.finish_non_exhaustive()
	}
}

#[cfg(test)]
mod tests {
	mod split_msg {
		use super::super::{split_msg_into_parts, MAX_MSG_LEN};
		const MSG_COUNT: usize = 3;

		#[test]
		fn empty_head_tail() {
			let head = String::new();

			let mut body = String::new();
			for _ in 0..MAX_MSG_LEN * MSG_COUNT {
				body.push('b');
			}

			let tail = String::new();

			let v = split_msg_into_parts(head, body, tail);
			assert_eq!(v.len(), MSG_COUNT);
		}

		#[test]
		fn long_head() {
			let mut head = String::new();
			for _ in 0..150 {
				head.push('h');
			}

			let mut body = String::new();
			for _ in 0..MAX_MSG_LEN * MSG_COUNT {
				body.push('b');
			}

			let tail = String::new();

			let v = split_msg_into_parts(head, body, tail);
			assert_eq!(v.len(), MSG_COUNT + 1);
		}

		#[test]
		fn with_tail_almost_fitting() {
			let head = String::new();

			let mut body = String::new();
			// body is 1 char from max msg len
			for _ in 0..MAX_MSG_LEN * MSG_COUNT - 1 {
				body.push('b');
			}

			let tail = "tt".to_owned(); // and tail is 2 char

			let v = split_msg_into_parts(head, body, tail);
			assert_eq!(v.len(), MSG_COUNT + 1); // tail shouldn't be split and thus should be put into it's own msg
		}
	}
}
