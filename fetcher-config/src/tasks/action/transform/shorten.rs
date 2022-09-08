/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::action::transform::Shorten as CoreShorten;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Shorten {
	len: usize,
}

impl Shorten {
	pub fn parse(self) -> CoreShorten {
		CoreShorten { len: self.len }
	}
}
