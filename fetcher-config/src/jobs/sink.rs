/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod telegram;

use self::telegram::Telegram;
use crate::{jobs::external_data::ProvideExternalData, Error};
use fetcher_core::sink;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Sink {
	Telegram(Telegram),
	Stdout,
}

impl Sink {
	pub fn parse(self, external: &dyn ProvideExternalData) -> Result<sink::Sink, Error> {
		Ok(match self {
			Sink::Telegram(x) => sink::Sink::Telegram(x.parse(external)?),
			Sink::Stdout => sink::Sink::Stdout(sink::Stdout {}),
		})
	}
}
