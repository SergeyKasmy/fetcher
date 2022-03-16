/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub mod message;
mod telegram;

pub use message::{Media, Message};
pub use telegram::Telegram;

use crate::error::Result;

#[derive(Debug)]
pub enum Sink {
	Telegram(Telegram),
}

impl Sink {
	#[allow(clippy::missing_errors_doc)] // TODO
	pub async fn send(&self, message: Message, tag: Option<&str>) -> Result<()> {
		match self {
			Self::Telegram(t) => t.send(message, tag).await,
		}
	}
}
