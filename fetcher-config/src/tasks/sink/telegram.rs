/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use crate::tasks::TaskSettings;
use crate::Error;
use fetcher_core::sink;

/// Refer to [`crate::sink::message::LinkLocation`]
#[derive(Deserialize, Serialize, Debug)]
// #[serde(rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum LinkLocation {
	PreferTitle,
	Bottom,
}

impl LinkLocation {
	pub fn parse(self) -> sink::telegram::LinkLocation {
		match self {
			LinkLocation::PreferTitle => sink::telegram::LinkLocation::PreferTitle,
			LinkLocation::Bottom => sink::telegram::LinkLocation::Bottom,
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
pub struct Telegram {
	chat_id: i64,
	link_location: LinkLocation,
}

impl Telegram {
	pub fn parse(self, settings: &dyn TaskSettings) -> Result<sink::Telegram, Error> {
		Ok(sink::Telegram::new(
			settings
				.telegram_bot_token()?
				.ok_or(Error::TelegramBotTokenMissing)?,
			self.chat_id,
			self.link_location.parse(),
		))
	}
}

