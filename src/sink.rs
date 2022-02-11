/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

mod message;
mod telegram;

pub use message::{Media, Message};
use serde::Deserialize;
pub use telegram::Telegram;

use crate::error::Result;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Sink {
	Telegram(Telegram),
}

impl Sink {
	pub async fn send(&self, message: Message) -> Result<()> {
		match self {
			Self::Telegram(t) => t.send(message).await,
		}
	}
}
