/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains [`Sink`] that can be used to consume a composed [`Message`],
//! as well as [`Message`](`message`) itself

pub mod message;

pub mod stdout;
pub mod telegram;

pub use self::{
	message::{Media, Message},
	stdout::Stdout,
	telegram::Telegram,
};
pub use crate::exec::Exec;

use crate::error::sink::Error as SinkError;
use async_trait::async_trait;
use std::fmt::Debug;

/// An async function that sends a message somewhere
#[async_trait]
pub trait Sink: Debug + Send + Sync {
	// pub trait Sink: Debug {
	/// Send the message with an optional tag (usually represented as a hashtag)
	async fn send(&self, message: Message, tag: Option<&str>) -> Result<(), SinkError>;
}
