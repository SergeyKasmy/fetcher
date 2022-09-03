/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::ConfigError;
use fetcher_core::action::transform::regex::Action as CoreAction;
use fetcher_core::action::transform::Regex as CoreRegex;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Regex {
	pub(crate) re: String,
	pub(crate) action: Action,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Action {
	Find,
	Extract { passthrough_if_not_found: bool },
}

impl Regex {
	pub(crate) fn parse(self) -> Result<CoreRegex, ConfigError> {
		Ok(CoreRegex::new(&self.re, self.action.parse())?)
	}
}

impl Action {
	pub(crate) fn parse(self) -> CoreAction {
		match self {
			Action::Find => CoreAction::Find,
			Action::Extract {
				passthrough_if_not_found,
			} => CoreAction::Extract {
				passthrough_if_not_found,
			},
		}
	}
}
