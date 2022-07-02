/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

mod telegram;

use serde::{Deserialize, Serialize};

use self::telegram::Telegram;
use super::DataSettings;
use crate::error::Result;
use crate::sink;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Sink {
	Telegram(Telegram),
	Stdout,
}

impl Sink {
	pub(crate) fn parse(self, settings: &DataSettings) -> Result<sink::Sink> {
		Ok(match self {
			Sink::Telegram(x) => sink::Sink::Telegram(x.parse(settings)?),
			Sink::Stdout => sink::Sink::Stdout(sink::Stdout {}),
		})
	}
}
