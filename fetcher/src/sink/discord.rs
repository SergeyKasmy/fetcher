/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Discord`] sink

use std::num::TryFromIntError;

use serenity::{
	all::{CreateEmbed, CreateEmbedFooter},
	builder::CreateMessage,
	http::Http as Bot,
	model::{
		channel::Message as DcMessage,
		id::{ChannelId, MessageId as DcMessageId, UserId},
	},
};

use super::{
	Sink,
	error::SinkError,
	message::{Media, Message, MessageId, length_limiter::MessageLengthLimiter},
};
use crate::utils::OptionExt;

// https://discord.com/developers/docs/resources/channel#create-message
const MAX_MSG_LEN: usize = 2000;
const MAX_EMBED_DESCIPTION_LEN: usize = 2000;

/// Discord sink. Supports both text channels and DMs with a user
#[derive(Debug)]
pub struct Discord {
	bot: Bot,
	target: TargetInner,
}

/// Target for the [`Discord`] sink where it sends message to
#[derive(Clone, Copy, Debug)]
pub enum Target {
	/// A text channel ID
	Channel(u64),

	/// A user ID, whose DMs to send messages into
	User(u64),
}

#[derive(Debug)]
enum TargetInner {
	Channel(ChannelId),
	User(UserId),
}

impl Discord {
	/// Create a new [`Discord`] sink. Needs a valid Discord bot `token` and a `target` where to send messages to
	#[must_use]
	pub fn new(token: &str, target: Target) -> Self {
		Self {
			bot: Bot::new(token),
			target: match target {
				Target::Channel(i) => TargetInner::Channel(i.into()),
				Target::User(i) => TargetInner::User(i.into()),
			},
		}
	}
}

impl Sink for Discord {
	async fn send(
		&self,
		msg: &Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError> {
		let mut last_message = reply_to.try_map(|msgid| {
			let dc_msgid = DcMessageId::from(u64::try_from(msgid.0)?);

			Ok::<_, TryFromIntError>(dc_msgid)
		})?;

		let Message {
			title,
			body,
			link,
			media,
		} = msg.clone(); // clone is to be able to include the message if an error happens

		// if the body of the message won't fit into an embed, then just send as regular messages
		if body.as_ref().map_or(0, |s| s.chars().count()) > MAX_EMBED_DESCIPTION_LEN {
			let mut head = title;

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

			let link = link.map(|s| s.to_string());

			let mut composed_msg = MessageLengthLimiter {
				head: head.as_deref(),
				body: body.as_deref(),
				tail: link.as_deref(),
			};

			while let Some(text) = composed_msg.split_at(MAX_MSG_LEN) {
				let msg = self
					.target
					.send_message(&self.bot, CreateMessage::new().content(&text))
					.await
					.map_err(|e| SinkError::Discord {
						source: e,
						msg: Box::new(text),
					})?;

				last_message = Some(msg.id);
			}
		}
		// send as an embed (much pretty, so wow!)
		else {
			let mut embed = CreateEmbed::new();

			if let Some(title) = title {
				embed = embed.title(title);
			}

			if let Some(body) = body {
				embed = embed.description(body);
			}

			if let Some(link) = link {
				embed = embed.url(link);
			}

			if let Some(tag) = tag {
				embed = embed.footer(CreateEmbedFooter::new(tag));
			}

			if let Some(media) = media {
				for media in media {
					if let Media::Photo(image) = media {
						embed = embed.image(image);
					}
				}
			}

			let msg = self
				.target
				.send_message(&self.bot, CreateMessage::new().embed(embed))
				.await
				.map_err(|e| SinkError::Discord {
					source: e,
					msg: Box::new(msg.clone()),
				})?;

			last_message = Some(msg.id);
		}

		// If it does, we should crash and think of a new solution anyways
		let msgid = last_message.map(|id| i64::try_from(id.get()).expect("not sure if Discord will ever return an ID that doesn't fit into MessageId. It shouldn't do that, probably...").into());
		Ok(msgid)
	}
}

impl TargetInner {
	async fn send_message(
		&self,
		bot: &Bot,
		message: CreateMessage,
	) -> Result<DcMessage, serenity::Error> {
		let msg = match self {
			TargetInner::Channel(chan) => chan.send_message(bot, message).await?,
			TargetInner::User(user) => {
				user.create_dm_channel(bot)
					.await?
					.send_message(bot, message)
					.await?
			}
		};

		Ok(msg)
	}
}
