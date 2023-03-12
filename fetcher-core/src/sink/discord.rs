/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Discord`] sink

use async_trait::async_trait;
use serde_json::{json, Value};
use serenity::http::Http as Bot;
use serenity::model::id::ChannelId;

use super::{
	error::SinkError,
	message::{Message, MessageId},
	Sink,
};

#[derive(Debug)]
pub struct Discord {
	bot: Bot,
	channel_id: ChannelId,
}

impl Discord {
	pub fn new(token: &str, channel_id: u64) -> Self {
		Self {
			bot: serenity::http::Http::new(token),
			channel_id: ChannelId(channel_id),
		}
	}
}

#[async_trait]
impl Sink for Discord {
	async fn send(
		&self,
		message: Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError> {
		let message = self
			.channel_id
			.send_message(&self.bot, |msg| {
				msg.content(message.body.unwrap());
				msg
			})
			.await
			.unwrap();

		Ok(None)
	}
}
