/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Telegram`] sink, as well as [`LinkLocation`] enum that specifies where to put a link in a telegram message

use crate::{
	sink::{
		Sink,
		error::SinkError,
		message::{Media, Message, MessageId, length_limiter::MessageLengthLimiter},
	},
	utils::OptionExt,
};

use async_trait::async_trait;
use std::{fmt::Debug, num::TryFromIntError, time::Duration};
use teloxide::{
	Bot, RequestError,
	adaptors::{Throttle, throttle::Limits},
	payloads::{SendMediaGroupSetters, SendMessageSetters},
	requests::{Request, Requester, RequesterExt},
	types::{
		ChatId, InputFile, InputMedia, InputMediaPhoto, InputMediaVideo, LinkPreviewOptions,
		Message as TelMessage, MessageId as TelMessageId, ParseMode, ReplyParameters,
	},
};
use tokio::time::sleep;

const MAX_TEXT_MSG_LEN: usize = 4096;
const MAX_MEDIA_MSG_LEN: usize = 1024;

const LINK_PREVIEW_DISABLED: LinkPreviewOptions = LinkPreviewOptions {
	is_disabled: true,
	url: None,
	prefer_small_media: false,
	prefer_large_media: false,
	show_above_text: false,
};

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
}

#[async_trait]
impl Sink for Telegram {
	/// Sends a message to a Telegram chat
	///
	/// # Errors
	/// * if Telegram returned an error
	/// * if there's no internet connection
	#[tracing::instrument(level = "debug", skip(message))]
	async fn send(
		&self,
		message: &Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError> {
		let reply_to = reply_to.try_map(|msgid| {
			let tel_msg_id = TelMessageId(msgid.0.try_into()?);
			Ok::<_, TryFromIntError>(tel_msg_id)
		})?;

		let (head, body, tail, media) = process_msg(message, tag, self.link_location);

		let processed_msg = MessageLengthLimiter {
			head: head.as_deref(),
			body: body.as_deref(),
			tail: tail.as_deref(),
		};

		let msg_id = self.send_processed(processed_msg, media, reply_to).await?;
		Ok(msg_id.map(|tel_msgid| i64::from(tel_msgid.0).into()))
	}
}

impl Telegram {
	// replace option with custom error
	async fn send_processed(
		&self,
		mut msg: MessageLengthLimiter<'_>,
		media: Option<&[Media]>,
		reply_to: Option<TelMessageId>,
	) -> Result<Option<TelMessageId>, SinkError> {
		let mut last_message = reply_to;

		// if the message contains media, send it and MAX_MEDIA_MSG_LEN chars first
		if let Some(media) = media {
			// send media only (i.e. without caption) if all the media wouldn't fit in a single message
			if media.len() > 10 {
				for ch in media.chunks(10) {
					let sent_msg = self.send_media(ch, None, last_message).await?;
					last_message = sent_msg.and_then(|v| v.first().map(|m| m.id));
				}
			} else {
				let media_caption = msg.split_at(MAX_MEDIA_MSG_LEN).expect(
					"should always return a valid split at least once since msg char len is > max_char_limit",
				);

				let sent_msg = self
					.send_media(media, Some(&media_caption), last_message)
					.await?;
				last_message = sent_msg.and_then(|v| v.first().map(|m| m.id));
			}
		}

		// send all remaining text in splits of MAX_TEXT_MSG_LEN
		// whether we sent a media message first is not important
		while let Some(text) = msg.split_at(MAX_TEXT_MSG_LEN) {
			let sent_msg = self.send_text(&text, last_message).await?;
			last_message = Some(sent_msg.id);
		}

		Ok(last_message)
	}
}

impl Telegram {
	#[tracing::instrument(level = "trace", skip(self, message))]
	async fn send_text(
		&self,
		message: &str,
		mut reply_to: Option<TelMessageId>,
	) -> Result<TelMessage, SinkError> {
		tracing::debug!(
			"About to send a text message with contents: {message:?}, replying to {reply_to:?}"
		);

		loop {
			tracing::info!("Sending text message");

			let send_msg_cmd = self
				.bot
				.send_message(self.chat_id, message)
				.parse_mode(ParseMode::Html)
				.link_preview_options(LINK_PREVIEW_DISABLED);

			let send_msg_cmd = if let Some(id) = reply_to {
				send_msg_cmd.reply_parameters(ReplyParameters::new(id))
			} else {
				send_msg_cmd
			};

			match send_msg_cmd.send().await {
				Ok(message) => return Ok(message),
				Err(e)
					if e.to_string()
						.to_lowercase()
						.contains("replied message not found") =>
				{
					tracing::warn!(
						"Message that should be replied to doesn't exist. Resending just as a regular message"
					);
					reply_to = None;
				}
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::error!(
						"Exceeded rate limit while using Throttle Bot adapter, this shouldn't happen... Retrying in {}s",
						retry_after.seconds()
					);
					sleep(retry_after.duration()).await;
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

	/// Returns None if Media couldn't be sent but it's Telegram's fault
	/// # Panics
	/// if media.len() is more than 10
	#[tracing::instrument(level = "trace", skip(self))]
	async fn send_media(
		&self,
		media: &[Media],
		mut caption: Option<&str>,
		mut reply_to: Option<TelMessageId>,
	) -> Result<Option<Vec<TelMessage>>, SinkError> {
		assert!(
			media.len() <= 10,
			"Trying to send more media items: {}, than max supported 10",
			media.len()
		);

		tracing::debug!(
			"About to send a media message with caption: {caption:?}, and media: {media:?}, replying to {reply_to:?}"
		);

		let media = media
			.iter()
			.map(|m| {
				macro_rules! input_media {
					// $type example: Photo
					// $full_type example: InputMediaPhoto
					($type:tt, $full_type:tt, $url:expr) => {{
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

			let msg_cmd = if let Some(id) = reply_to {
				msg_cmd.reply_parameters(ReplyParameters::new(id))
			} else {
				msg_cmd
			};

			// don't forget to return from a branch, dummy, otherwise you'll end up in an infinite loop
			#[expect(clippy::redundant_else, reason = "improves control flow visualization")]
			match msg_cmd.send().await {
				Ok(messages) => return Ok(Some(messages)),
				Err(e)
					if e.to_string()
						.to_lowercase()
						.contains("failed to get http url content") =>
				{
					if retry_counter > 5 {
						tracing::warn!("Telegram failed to get URL content too many times");

						if let Some(caption) = caption {
							tracing::info!("Sending the message as pure text...");

							let msg = self.send_text(caption, reply_to).await?;

							return Ok(Some(vec![msg]));
						} else {
							tracing::warn!("There's no text to send, skipping this message...");
							return Ok(None);
						}
					}

					tracing::warn!("Telegram failed to get URL content. Retrying in 30 seconds");
					sleep(Duration::from_secs(30)).await;

					retry_counter += 1;
				}
				Err(e)
					if e.to_string()
						.to_lowercase()
						.contains("wrong file identifier/http url specified") =>
				{
					// TODO: reupload the image manually if this happens
					if let Some(caption) = caption {
						tracing::warn!(
							"Telegram disliked the media URL (\"Wrong file identifier/HTTP URL specified\"), sending the message as pure text"
						);
						let msg = self.send_text(caption, reply_to).await?;

						return Ok(Some(vec![msg]));
					} else {
						tracing::warn!(
							"Telegram disliked the media URL (\"Wrong file identifier/HTTP URL specified\") but the caption was empty, skipping..."
						);
						return Ok(None);
					}
				}
				Err(e)
					if e.to_string()
						.to_lowercase()
						.contains("wrong type of the web page content") =>
				{
					// TODO: reupload the image manually if this happens
					if let Some(caption) = caption {
						tracing::warn!(
							"Telegram disliked the media URL (\"Wrong type of the web page content\"), sending the message as pure text"
						);
						let msg = self.send_text(caption, reply_to).await?;

						return Ok(Some(vec![msg]));
					} else {
						tracing::warn!(
							"Telegram disliked the media URL (\"Wrong type of the web page content\") but the caption was empty, skipping..."
						);
						return Ok(None);
					}
				}
				Err(e)
					if e.to_string()
						.to_lowercase()
						.contains("replied message not found") =>
				{
					tracing::warn!(
						"Message that should be replied to doesn't exist. Resending just as a regular message"
					);
					reply_to = None;
				}
				Err(RequestError::RetryAfter(retry_after)) => {
					tracing::error!(
						"Exceeded rate limit while using Throttle Bot adapter, this shouldn't happen... Retrying in {}s",
						retry_after.seconds()
					);
					sleep(retry_after.duration()).await;
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

type HeadBodyTailMedia<'a> = (
	Option<String>,
	Option<String>,
	Option<String>,
	Option<&'a [Media]>,
);

// format and sanitize all message fields. Returns (head, body, tail, media)
fn process_msg<'a>(
	msg: &'a Message,
	tag: Option<&str>,
	link_location: LinkLocation,
) -> HeadBodyTailMedia<'a> {
	let Message {
		title,
		body,
		link,
		media,
	} = msg;

	// escape title and body
	let title = title.as_deref().map(teloxide::utils::html::escape);
	let body = body.as_deref().map(teloxide::utils::html::escape);

	// put the link into the message
	let (mut head, tail) = match (title, link) {
		// if title and link are both present
		(Some(title), Some(link)) => match link_location {
			// and the link should be in the title, then combine them
			LinkLocation::PreferTitle => (Some(format!("<a href=\"{link}\">{title}</a>")), None),
			// and it should be at the bottom, return both separately
			LinkLocation::Bottom => (Some(title), Some(format!("<a href=\"{link}\">Link</a>"))),
		},
		// if only the title is present, just return itself
		(Some(title), None) => (Some(title), None),
		// and if only the link is present, but it at the bottom of the message, even if it should try to be in the title
		(None, Some(link)) => (None, Some(format!("<a href=\"{link}\">Link</a>"))),
		(None, None) => (None, None),
	};

	// add tag as a hashtag on top of the message
	if let Some(tag) = tag {
		let tag = tag.replace(
			|c| match c {
				'_' => false,
				c if c.is_alphabetic() || c.is_ascii_digit() => false,
				_ => true,
			},
			"_",
		);

		head = Some({
			let mut head = head
				// add more padding between tag and title if both are present
				.map(|mut s| {
					s.insert(0, '\n');
					s
				})
				.unwrap_or_default();

			head.insert_str(0, &format!("#{tag}\n"));
			head
		});
	}

	(head, body, tail, media.as_deref())
}

impl Debug for Telegram {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Telegram")
			.field("chat_id", &self.chat_id)
			.field("link_location", &self.link_location)
			.finish_non_exhaustive()
	}
}
