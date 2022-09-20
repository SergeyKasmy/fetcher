/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod telegram;

use serde::{Deserialize, Serialize};

use self::telegram::Telegram;
use super::TaskSettings;
use crate::Error;
use fetcher_core::sink;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum Sink {
	Telegram(Telegram),
	Stdout,
}

impl Sink {
	pub fn parse(self, settings: &dyn TaskSettings) -> Result<sink::Sink, Error> {
		Ok(match self {
			Sink::Telegram(x) => sink::Sink::Telegram(x.parse(settings)?),
			Sink::Stdout => sink::Sink::Stdout(sink::Stdout {}),
		})
	}
}
