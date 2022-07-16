/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod message;
pub(crate) mod stdout;
pub mod telegram;

pub use message::{Media, Message};
pub use stdout::Stdout;
pub use telegram::Telegram;

use crate::error::sink::Error as SinkError;

#[derive(Debug)]
pub enum Sink {
	Telegram(Telegram),
	Stdout(Stdout),
	Null,
}

impl Sink {
	#[allow(clippy::missing_errors_doc)] // TODO
	pub async fn send(&self, message: Message, tag: Option<&str>) -> Result<(), SinkError> {
		match self {
			Self::Telegram(t) => t.send(message, tag).await,
			Self::Stdout(s) => s.send(message, tag).await,
			Self::Null => Ok(()),
		}
	}
}
