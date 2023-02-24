/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use fetcher_core::action::transform::Use as CUse;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Use {
	pub field: Field,
	pub as_field: Field,
}

impl Use {
	pub fn parse(self) -> CUse {
		CUse {
			field: self.field.parse(),
			as_field: self.as_field.parse(),
		}
	}
}
