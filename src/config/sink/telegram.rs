/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};
use teloxide::types::ChatId;

use crate::{error::Result, sink};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Telegram {
	chat_id: ChatId,
}

impl Telegram {
	pub(crate) fn parse(self) -> Result<sink::Telegram> {
		let chat_id = match std::env::var("FETCHER_DEBUG_CHAT_ID") {
			Ok(s) => ChatId::try_from(s).expect("Invalid chat id in FETCHER_DEBUG_CHAT_ID"),
			_ => self.chat_id,
		};
		Ok(sink::Telegram::new(settings::telegram()?, chat_id))
	}
}
