/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod take;

use self::take::Take;
use fetcher_core::action::filter::Kind as CoreFilterKind;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Filter {
	ReadFilter,
	Take(Take),
}

impl Filter {
	pub fn parse(self) -> CoreFilterKind {
		match self {
			Self::ReadFilter => unreachable!(),
			Self::Take(x) => CoreFilterKind::Take(x.parse()),
		}
	}
}
