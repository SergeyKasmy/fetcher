/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod discord;
pub mod exec;
pub mod telegram;

pub use self::{discord::Discord, exec::Exec, telegram::Telegram};

use crate::{jobs::external_data::ProvideExternalData, Error};
use fetcher_core::sink::{Sink as CSink, Stdout as CStdout};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, PartialEq, Default, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Sink {
	Telegram(Telegram),
	Discord(Discord),
	Exec(Exec),
	#[default]
	Stdout,
}

impl Sink {
	pub fn parse<D>(self, external: &D) -> Result<Box<dyn CSink>, Error>
	where
		D: ProvideExternalData + ?Sized,
	{
		Ok(match self {
			Self::Telegram(x) => Box::new(x.parse(external)?),
			Self::Discord(x) => Box::new(x.parse(external)?),
			Self::Exec(x) => Box::new(x.parse()),
			Self::Stdout => Box::new(CStdout {}),
		})
	}
}
