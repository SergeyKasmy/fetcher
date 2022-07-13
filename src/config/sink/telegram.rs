/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::{config::DataSettings, error::config::Error as ConfigError, sink};

/// Refer to [`crate::sink::message::LinkLocation`]
#[derive(Deserialize, Serialize, Debug)]
// #[serde(rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum LinkLocation {
	PreferTitle,
	Bottom,
}

impl LinkLocation {
	pub(crate) fn parse(self) -> sink::telegram::LinkLocation {
		match self {
			LinkLocation::PreferTitle => sink::telegram::LinkLocation::PreferTitle,
			LinkLocation::Bottom => sink::telegram::LinkLocation::Bottom,
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
pub(crate) struct Telegram {
	chat_id: i64,
	link_location: LinkLocation,
}

impl Telegram {
	pub(crate) fn parse(self, settings: &DataSettings) -> Result<sink::Telegram, ConfigError> {
		// let chat_id = match std::env::var("FETCHER_DEBUG_CHAT_ID") {
		// 	Ok(s) => s
		// 		.parse::<i64>()
		// 		.map_err(|_| Error::Other("Invalid chat id in FETCHER_DEBUG_CHAT_ID".to_owned()))?,
		// 	_ => self.chat_id,
		// };
		Ok(sink::Telegram::new(
			settings
				.telegram
				.as_ref()
				.cloned()
				.ok_or(ConfigError::TelegramBotTokenMissing)?,
			self.chat_id,
			self.link_location.parse(),
		))
	}
}
