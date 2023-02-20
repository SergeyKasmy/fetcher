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

pub use crate::exec::Exec;
pub use message::{Media, Message};
pub use stdout::Stdout;
pub use telegram::Telegram;

use crate::error::sink::Error as SinkError;

/// All available sinks
#[derive(Debug)]
pub enum Sink {
	/// Refer to [`Telegram`] docs
	Telegram(Telegram),
	/// Refer to [`Exec`] docs
	Exec(Exec),
	/// Refer to [`Stdout`] docs
	Stdout(Stdout),
}

impl Sink {
	/// Send a message with an optional tag to the sink
	///
	/// # Errors
	/// if there was an error sending the message
	pub async fn send(&self, message: Message, tag: Option<&str>) -> Result<(), SinkError> {
		match self {
			Self::Telegram(t) => t.send(message, tag).await,
			Self::Exec(x) => x.send(message).await.map_err(Into::into),
			Self::Stdout(s) => s.send(message, tag).await,
		}
	}
}
