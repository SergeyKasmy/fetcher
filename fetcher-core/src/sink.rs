/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Sink`] that can be used to consume a composed [`Message`],
//! as well as the [`message`] module itself

pub mod message;

pub mod discord;
pub mod stdout;
pub mod telegram;

pub mod error;

pub use self::{discord::Discord, stdout::Stdout, telegram::Telegram};
pub use crate::exec::Exec;

use self::{
	error::SinkError,
	message::{Message, MessageId},
};

use async_trait::async_trait;
use std::fmt::Debug;

/// An async function that sends a message somewhere
#[async_trait]
pub trait Sink: Debug + Send + Sync {
	/// Send the message with an optional tag (usually represented as a hashtag)
	async fn send(
		&self,
		message: &Message,
		reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError>;
}
