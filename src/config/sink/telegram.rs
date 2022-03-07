/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;
use teloxide::types::ChatId;

use crate::{error::Result, settings, sink};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Telegram {
	chat_id: ChatId,
}

impl Telegram {
	pub(crate) fn parse(self) -> Result<sink::Telegram> {
		Ok(sink::Telegram::new(settings::telegram()?, self.chat_id))
	}
}
