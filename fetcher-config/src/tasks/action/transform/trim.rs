/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::action::transform::Trim as CoreTrim;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Trim {
	Title,
	Body,
	All,
}

impl Trim {
	pub fn parse(self) -> CoreTrim {
		match self {
			Self::Title => CoreTrim::Title,
			Self::Body => CoreTrim::Body,
			Self::All => CoreTrim::All,
		}
	}
}
