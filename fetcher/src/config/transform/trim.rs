/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::transform::Trim as CoreTrim;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub(crate) enum Trim {
	Title,
	Body,
	All,
}

impl Trim {
	pub(crate) fn parse(self) -> CoreTrim {
		match self {
			Self::Title => CoreTrim::Title,
			Self::Body => CoreTrim::Body,
			Self::All => CoreTrim::All,
		}
	}
}
