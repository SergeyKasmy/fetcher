/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
	jobs::external_data::{ExternalDataResult, ProvideExternalData},
	Error as ConfigError,
};
use fetcher_core::sink::discord::{Discord as CDiscord, Target as CTarget};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Discord {
	pub target: Target,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Target {
	User(u64),
	Channel(u64),
}

impl Discord {
	pub fn parse<D>(self, external: &D) -> Result<CDiscord, ConfigError>
	where
		D: ProvideExternalData + ?Sized,
	{
		let token = match external.discord_bot_token() {
			ExternalDataResult::Ok(v) => v,
			ExternalDataResult::Unavailable => return Err(ConfigError::DiscordBotTokenMissing),
			ExternalDataResult::Err(e) => return Err(e.into()),
		};

		Ok(CDiscord::new(&token, self.target.parse()))
	}
}

impl Target {
	pub fn parse(self) -> CTarget {
		match self {
			Target::User(i) => CTarget::User(i),
			Target::Channel(i) => CTarget::Channel(i),
		}
	}
}
