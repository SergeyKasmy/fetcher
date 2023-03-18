/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Field;
use crate::error::Error as ConfigError;
use fetcher_core::action::filter::Contains as CContains;

use serde::{Deserialize, Serialize};

// TODO: use a hashmap
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Contains {
	pub field: Field,
	pub re: String,
}

impl Contains {
	pub fn parse(self) -> Result<CContains, ConfigError> {
		CContains::new(&self.re, self.field.parse()).map_err(Into::into)
	}
}
