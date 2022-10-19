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

use std::{fmt::Debug, time::Duration};
use teloxide::{
	adaptors::{throttle::Limits, Throttle},
	payloads::{SendMediaGroupSetters, SendMessageSetters},
	requests::{Request, Requester, RequesterExt},
	types::{
		ChatId, InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, Message as TelMessage,
		MessageId, ParseMode,
	},
	ApiError, Bot, RequestError,
};
use url::Url;

const MAX_TEXT_MSG_LEN: usize = 4096;
const MAX_MEDIA_MSG_LEN: usize = 1024;

/// Telegram sink. Supports text and media messages and embeds text into media captions if present. Automatically splits the text into separate messages if it's too long
pub struct Telegram {
	bot: Throttle<Bot>,
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
			bot: Bot::new(token).throttle(Limits::default()),
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
			"Processing message: title: {title:?}, body len: {}, link: {}, media count: {}",
			body.as_ref().map_or(0, String::len),
			link.is_some(),
			media.as_ref().map_or(0, Vec::len),
		);

		let body = body.map(|s| teloxide::utils::html::escape(&s));
		let (head, tail) = format_head_tail(
			title.map(|s| teloxide::utils::html::escape(&s)),
			link,
			tag,
			self.link_location,
		);

		let text = MsgParts {
			head: head.as_deref(),
			body: body.as_deref(),
			tail: tail.as_deref(),
		};

		// fugure out if additional newline charaters should be added
		// and include them in calculations on whether the message will end up too long.
		// add newline after head if head.is some and either body or tail is some
		let should_insert_newline_after_head = head.is_some() && (body.is_some() || tail.is_some());
		let should_insert_newline_after_body = body.is_some() && tail.is_some();

		let max_char_limit = if media.is_some() {
			MAX_MEDIA_MSG_LEN
		} else {
			MAX_TEXT_MSG_LEN
		};

		// if total single message char len would be bigger than max_char_limit (depending on whether the message contains media)
		if head.as_ref().map_or(0, |s| s.chars().count())
			+ body.as_ref().map_or(0, |s| s.chars().count())
			+ tail.as_ref().map_or(0, |s| s.chars().count())
			+ usize::from(should_insert_newline_after_head)
			+ usize::from(should_insert_newline_after_body)
			> max_char_limit
		{
			self.process_long_message(text, media.as_deref()).await?;
		} else {
			self.process_short_message(
				text,
				media.as_deref(),
				should_insert_newline_after_head,
				should_insert_newline_after_body,
			)
			.await?;
		}

		Ok(())
	}

	async fn process_long_message(
		&self,
		mut text: MsgParts<'_>,
		media: Option<&[Media]>,
	) -> Result<(), SinkError> {
		let mut previous_message = None;

		// if the message contains media, send it and MAX_MEDIA_MSG_LEN chars first
		if let Some(media) = media {
			// send media only (i.e. without caption) if all the media wouldn't fit in a single message
			if media.len() > 10 {
				for ch in media.chunks(10) {
					let sent_msg = self
						.send_media_with_reply_id(ch, None, previous_message)
						.await?;
					previous_message = Some(sent_msg[0].id);
				}
			} else {
				let media_caption = text
						.split_msg_at(MAX_MEDIA_MSG_LEN)
						.expect("should always return a valid split at least once since msg char len is > max_char_limit");

				let sent_msg = self
					.send_media_with_reply_id(media, Some(&media_caption), previous_message)
					.await?;
				previous_message = Some(sent_msg[0].id);
			}
		}

		// send all remaining text in splits of MAX_TEXT_MSG_LEN
		// whether we sent a media message first is not important
		while let Some(text) = text.split_msg_at(MAX_TEXT_MSG_LEN) {
			let sent_msg = self
				.send_text_with_reply_id(&text, previous_message)
				.await?;
			previous_message = Some(sent_msg.id);
		}

		Ok(())
	}

	async fn process_short_message(
		&self,
		text: MsgParts<'_>,
		media: Option<&[Media]>,

		// passthrough these to avoid recalculation or desync with the previous calculations
		// even though they do make this fn signature uglier
		should_insert_newline_after_head: bool,
		should_insert_newline_after_body: bool,
	) -> Result<(), SinkError> {
		macro_rules! newline_if {
			($bool:expr) => {
				if $bool {
					"\n"
				} else {
					""
				}
			};
		}

		let MsgParts { head, body, tail } = text;

		let text = format!(
			"{}{}{}{}{}",
			head.unwrap_or_default(),
			newline_if!(should_insert_newline_after_head),
			body.unwrap_or_default(),
			newline_if!(should_insert_newline_after_body),
			tail.unwrap_or_default()
		);

		let text = if text.trim().is_empty() {
			None
		} else {
			Some(text)
		};

		if let Some(media) = media {
			self.send_media(media, text.as_deref()).await?;
		} else {
			match text {
				Some(text) => {
					self.send_text(&text).await?;
				}
				None => {
					tracing::warn!("Skipping sending completely empty Telegram text message");
				}
			}
		}

		Ok(())
	}

	async fn send_text(&self, message: &str) -> Result<TelMessage, SinkError> {
		self.send_text_with_reply_id(message, None).await
	}

	async fn send_text_with_reply_id(
		&self,
		message: &str,
		reply_to_msg_id: Option<MessageId>,
	) -> Result<TelMessage, SinkError> {
		tracing::trace!("About to send a text message with contents: {message:?}");
		loop {
			tracing::info!("Sending text message");

			let send_msg_cmd = self
				.bot
				.send_message(self.chat_id, message)
				.parse_mode(ParseMode::Html)
				.disable_web_page_preview(true);

			let send_msg_cmd = if let Some(id) = reply_to_msg_id {
				send_msg_cmd.reply_to_message_id(id)
			} else {
				send_msg_cmd
			};

			match send_msg_cmd.send().await {
				Ok(message) => return Ok(message),
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::error!(
						"Exceeded rate limit while using Throttle Bot adapter, this shouldn't happen... Retrying in {}s",
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

	async fn send_media(
		&self,
		media: &[Media],
		caption: Option<&str>,
	) -> Result<Vec<TelMessage>, SinkError> {
		self.send_media_with_reply_id(media, caption, None).await
	}

	/// # Panics
	/// if media.len() is more than 10
	async fn send_media_with_reply_id(
		&self,
		media: &[Media],
		mut caption: Option<&str>,
		reply_to_msg_id: Option<MessageId>,
	) -> Result<Vec<TelMessage>, SinkError> {
		assert!(
			media.len() <= 10,
			"Trying to send more media items: {}, than max supported 10",
			media.len()
		);

		tracing::trace!(
			"About to send a media message with caption: {caption:?}, and media: {media:?}"
		);

		let media = media
			.iter()
			.map(|m| {
				macro_rules! input_media {
					($type:tt, $full_type:tt, $url:expr) => {{
						// $type example: Photo
						// $full_type example: InputMediaPhoto

						let input_media = $full_type::new(InputFile::url($url.clone()))
							.parse_mode(ParseMode::Html);

						let input_media = if let Some(caption) = caption.take() {
							input_media.caption(caption)
						} else {
							input_media
						};

						InputMedia::$type(input_media)
					}};
				}

				match m {
					Media::Photo(url) => input_media!(Photo, InputMediaPhoto, url),
					Media::Video(url) => input_media!(Video, InputMediaVideo, url),
				}
			})
			.collect::<Vec<_>>();

		// number of "failed to get url content" error retried tries
		let mut retry_counter = 0;

		loop {
			tracing::info!("Sending media message");

			let msg_cmd = self.bot.send_media_group(self.chat_id, media.clone());

			let msg_cmd = if let Some(id) = reply_to_msg_id {
				msg_cmd.reply_to_message_id(id)
			} else {
				msg_cmd
			};

			match msg_cmd.send().await {
				Ok(messages) => return Ok(messages),
				Err(e @ RequestError::Api(ApiError::FailedToGetUrlContent)) => {
					if retry_counter > 5 {
						tracing::error!(
							"Telegram failed tp get URL content too many times, exiting..."
						);

						return Err(SinkError::Telegram {
							source: e,
							msg: Box::new(media),
						});
					}
					tracing::warn!("Telegram failed to get URL content. Retrying in 30 seconds");
					tokio::time::sleep(Duration::from_secs(30)).await;

					retry_counter += 1;
				}
				Err(RequestError::Api(ApiError::WrongFileIdOrUrl)) => {
					// TODO: reupload the image manually if this happens
					if let Some(caption) = caption {
						tracing::warn!("Telegram disliked the media URL (\"Bad Request: wrong file identifier/HTTP URL specified\"), sending the message as pure text");
						self.send_text(caption).await?;
					} else {
						tracing::warn!("Telegram disliked the media URL (\"Bad Request: wrong file identifier/HTTP URL specified\") but the caption was empty, skipping...");
					}
				}
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::error!(
						"Exceeded rate limit while using Throttle Bot adapter, this shouldn't happen... Retrying in {}s",
						retry_after.as_secs()
					);
					tokio::time::sleep(retry_after).await;
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

fn format_head_tail(
	title: Option<String>,
	link: Option<Url>,
	tag: Option<&str>,
	link_location: LinkLocation,
) -> (Option<String>, Option<String>) {
	let (mut head, tail) = match (title, link) {
		// if title and link are both present
		(Some(title), Some(link)) => match link_location {
			// and the link should be in the title, then combine them
			LinkLocation::PreferTitle => (Some(format!("<a href=\"{link}\">{title}</a>")), None),
			// even it should be at the bottom, return both separately
			LinkLocation::Bottom => (Some(title), Some(format!("<a href=\"{link}\">Link</a>"))),
		},
		// if only the title is presend, just print itself with an added newline
		(Some(title), None) => (Some(title), None),
		// and if only the link is present, but it at the bottom of the message, even if it should try to be in the title
		(None, Some(link)) => (None, Some(format!("<a href=\"{link}\">Link</a>"))),
		(None, None) => (None, None),
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

		let mut head_wip = head.unwrap_or_default();
		head_wip.insert_str(0, &format!("#{tag}\n"));

		head = Some(head_wip);
	}

	(head, tail)
}

/// All parts of a message. `head` and `tail` can't be split over several messages, `body` can
#[derive(Debug)]
struct MsgParts<'a> {
	head: Option<&'a str>,
	body: Option<&'a str>,
	tail: Option<&'a str>,
}

impl MsgParts<'_> {
	/// returns head/body/tail as a formatted message at most `len` long.
	/// Acts similarly to a fused iterator and returns Some(msg) until every part of the message has been sent, afterwards always returns None
	fn split_msg_at(&mut self, len: usize) -> Option<String> {
		if self.head.is_none() && self.body.is_none() && self.tail.is_none() {
			return None;
		}

		// make sure the entire head or tail can fit into the requested split
		assert!(len >= self.head.map_or(0, |s| s.chars().count()));
		assert!(len >= self.tail.map_or(0, |s| s.chars().count()));

		let mut split_part = String::with_capacity(len);

		// put the entire head into the split
		if let Some(head) = self.head.take() {
			split_part.push_str(head);
		}

		if let Some(body) = self.body.take() {
			// find out how much space has remained for the body
			let space_left_for_body = len.checked_sub(split_part.chars().count()).expect("only the head should've been pushed to the split and we asserted that it isn't longer than len");

			// find the index at which point the body no longer fits into the split
			let body_fits_till = body
				.char_indices()
				.nth(space_left_for_body)
				.map_or_else(|| body.len(), |(idx, _)| idx);

			// mark if we should add a newline character and leave some space for it
			let (body_fits_till, add_newline) = if split_part.is_empty() {
				(body_fits_till, false)
			} else {
				(body_fits_till - 1, true)
			};

			// if at least some of the body does fit
			if body_fits_till > 0 {
				// insert a new line to separate body from everything else
				if add_newline {
					split_part.push('\n');
				}

				split_part.push_str(&body[..body_fits_till]);

				// if there are some bytes remaining in the body, put them back into itself
				let remaining_body = &body[body_fits_till..];
				if !remaining_body.is_empty() {
					self.body = Some(remaining_body);
				}
			}
		}

		// tail
		{
			let tail_len = self.tail.map_or(0, |s| s.chars().count());
			// mark if we should add a newline character and leave some space for it
			let (tail_len, add_newline) = if split_part.is_empty() {
				(tail_len, false)
			} else {
				(tail_len - 1, true)
			};

			// add the tail if it can still fit into the split
			if split_part.chars().count() > tail_len {
				if let Some(tail) = self.tail.take() {
					// insert a newline to separate tail from everything else
					if add_newline {
						split_part.push('\n');
					}

					split_part.push_str(tail);
				}
			}
		}

		let split_part_chars = split_part.chars().count();
		tracing::trace!("Final part len: {}", split_part_chars);
		// make sure we haven't crossed our character limit
		assert!(split_part_chars <= len);

		Some(split_part)
	}
}

impl Debug for Telegram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Telegram")
			.field("chat_id", &self.chat_id)
			.field("link_location", &self.link_location)
			.finish_non_exhaustive()
	}
}

/*
// TODO: rewrite these outdated tests
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
*/
