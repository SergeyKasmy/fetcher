/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use crate::tasks::external_data::ExternalData;
use crate::Error as ConfigError;
use fetcher_core::sink::telegram::LinkLocation as CLinkLocation;
use fetcher_core::sink::Telegram as CTelegram;

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Telegram {
	chat_id: i64,
	link_location: Option<LinkLocation>,
}

/// Refer to [`crate::sink::message::LinkLocation`]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum LinkLocation {
	PreferTitle,
	Bottom,
}

impl Telegram {
	pub fn parse(self, external: &dyn ExternalData) -> Result<CTelegram, ConfigError> {
		Ok(CTelegram::new(
			external
				.telegram_bot_token()?
				.ok_or(ConfigError::TelegramBotTokenMissing)?,
			self.chat_id,
			self.link_location
				.map_or(CLinkLocation::PreferTitle, LinkLocation::parse),
		))
	}
}

impl LinkLocation {
	pub fn parse(self) -> CLinkLocation {
		match self {
			LinkLocation::PreferTitle => CLinkLocation::PreferTitle,
			LinkLocation::Bottom => CLinkLocation::Bottom,
		}
	}
}
