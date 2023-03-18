/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::action::filter::take::{Take as CTake, TakeFrom as CTakeFrom};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Take {
	from: TakeFrom,
	num: usize,
}

// TODO: rename to new and old
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TakeFrom {
	Beginning,
	End,
}

impl Take {
	pub fn parse(self) -> CTake {
		CTake {
			from: self.from.parse(),
			num: self.num,
		}
	}
}

impl TakeFrom {
	pub fn parse(self) -> CTakeFrom {
		match self {
			TakeFrom::Beginning => CTakeFrom::Beginning,
			TakeFrom::End => CTakeFrom::End,
		}
	}
}
