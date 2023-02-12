/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::action::filter::take as core_take;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Take {
	from: TakeFrom,
	num: usize,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TakeFrom {
	Beginning,
	End,
}

impl Take {
	pub fn parse(self) -> core_take::Take {
		core_take::Take {
			from: self.from.parse(),
			num: self.num,
		}
	}
}

impl TakeFrom {
	pub fn parse(self) -> core_take::TakeFrom {
		match self {
			TakeFrom::Beginning => core_take::TakeFrom::Beginning,
			TakeFrom::End => core_take::TakeFrom::End,
		}
	}
}
