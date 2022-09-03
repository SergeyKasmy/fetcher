/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::transform::take as core_take;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Take {
	from: TakeFrom,
	num: usize,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TakeFrom {
	Beginning,
	End,
}

impl Take {
	pub(crate) fn parse(self) -> core_take::Take {
		core_take::Take {
			from: self.from.parse(),
			num: self.num,
		}
	}
}

impl TakeFrom {
	pub(crate) fn parse(self) -> core_take::TakeFrom {
		match self {
			TakeFrom::Beginning => core_take::TakeFrom::Beginning,
			TakeFrom::End => core_take::TakeFrom::End,
		}
	}
}
