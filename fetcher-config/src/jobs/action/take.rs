/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::action::filter::take::{Take as CTake, TakeFrom as CTakeFrom};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Take(#[serde(with = "crate::serde_extentions::tuple")] pub Inner);

#[derive(Clone, Debug)]
pub struct Inner {
	pub which: TakeWhich,
	pub num: usize,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TakeWhich {
	FromNewest,
	FromOldest,
}

impl Take {
	pub fn parse(self) -> CTake {
		CTake {
			from: self.0.which.parse(),
			num: self.0.num,
		}
	}
}

impl TakeWhich {
	pub fn parse(self) -> CTakeFrom {
		match self {
			TakeWhich::FromNewest => CTakeFrom::Beginning,
			TakeWhich::FromOldest => CTakeFrom::End,
		}
	}
}

impl<'a> From<&'a Inner> for (&'a TakeWhich, &'a usize) {
	fn from(Inner { which, num }: &'a Inner) -> Self {
		(which, num)
	}
}

impl From<(TakeWhich, usize)> for Inner {
	fn from((which, num): (TakeWhich, usize)) -> Self {
		Self { which, num }
	}
}
