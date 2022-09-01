/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::transform::Regex as CoreRegex;

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Regex {
	pub(crate) re: String,
	pub(crate) passthrough_if_not_found: bool,
}

impl Regex {
	pub(crate) fn parse(self) -> Result<CoreRegex, ConfigError> {
		Ok(CoreRegex::new(&self.re, self.passthrough_if_not_found)?)
	}
}
