/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Discord`] sink

use async_trait::async_trait;
use serenity::{
	builder::CreateMessage,
	http::Http as Bot,
	model::{
		channel::Message as DcMessage,
		id::{ChannelId, MessageId as DcMessageId, UserId},
	},
};

use super::{
	error::SinkError,
	message::{Message, MessageId},
	Sink,
};
use crate::utils::OptionExt;

// https://discord.com/developers/docs/resources/channel#create-message
const MAX_MSG_LEN: usize = 2000;

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

#[async_trait]
impl Sink for Discord {
	async fn send(
		&self,
		msg: Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError> {
		let (mut composed_msg, _media) = msg.compose(tag, None);

		let mut last_message = reply_to.try_map(|msgid| {
			let dc_msgid =
				DcMessageId::from(u64::try_from(msgid.0).map_err(SinkError::InvalidMessageIdType)?);

			Ok::<_, SinkError>(dc_msgid)
		})?;

		while let Some(text) = composed_msg.split_at(MAX_MSG_LEN) {
			let msg = self
				.target
				.send_message(&self.bot, |msg| {
					msg.content(&text);
					msg
				})
				.await
				.map_err(|e| SinkError::Discord {
					source: e,
					msg: Box::new(text),
				})?;

			last_message = Some(msg.id);
		}

		// If it does, we should crash and think of a new solution anyways
		let msgid = last_message.map(|id| i64::try_from(id.0).expect("not sure if Discord will ever return an ID that doesn't fit into MessageId. It shouldn't do that, probably...").into());

		Ok(msgid)
	}
}

impl TargetInner {
	async fn send_message<'a, F>(&self, bot: &Bot, f: F) -> Result<DcMessage, serenity::Error>
	where
		F: for<'b> FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
	{
		let msg = match self {
			TargetInner::Channel(chan) => chan.send_message(bot, f).await?,
			TargetInner::User(user) => {
				user.create_dm_channel(bot)
					.await?
					.send_message(bot, f)
					.await?
			}
		};

		Ok(msg)
	}
}
