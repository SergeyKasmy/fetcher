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

// https://discord.com/developers/docs/resources/channel#create-message
const MAX_MSG_LEN: usize = 2000;

#[derive(Debug)]
pub struct Discord {
	bot: Bot,
	target: TargetInner,
}

#[derive(Debug)]
pub enum Target {
	Channel(u64),
	User(u64),
}

#[derive(Debug)]
enum TargetInner {
	Channel(ChannelId),
	User(UserId),
}

impl Discord {
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
		// let message = self
		// 	.channel_id
		// 	.send_message(&self.bot, |msg| {
		// 		msg.content(message.body.unwrap());
		// 		msg
		// 	})
		// 	.await
		// 	.unwrap();

		// Ok(None)

		let (mut composed_msg, _media) = msg.compose(tag, None);

		let mut last_message = reply_to.map(|id| DcMessageId::from(u64::try_from(id.0).unwrap()));

		while let Some(text) = composed_msg.split_at(MAX_MSG_LEN) {
			let msg = self
				.target
				.send_message(&self.bot, |msg| {
					msg.content(text);
					msg
				})
				.await;

			last_message = Some(msg.id);
		}

		Ok(last_message.map(|id| i64::try_from(id.0).unwrap().into()))
	}
}

impl TargetInner {
	async fn send_message<'a, F>(&self, bot: &Bot, f: F) -> DcMessage
	where
		F: for<'b> FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>,
	{
		match self {
			TargetInner::Channel(chan) => chan.send_message(bot, f).await.unwrap(),
			TargetInner::User(user) => user
				.create_dm_channel(bot)
				.await
				.unwrap()
				.send_message(bot, f)
				.await
				.unwrap(),
		}
	}
}
