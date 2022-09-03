/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod filter;
pub mod transform;

use self::filter::Filter;
use self::transform::Transform;
use crate::error::ConfigError;
use fetcher_core::action::Action as CoreAction;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum Action {
	Filter(Filter),
	Transform(Transform),
}

impl Action {
	pub fn parse(self) -> Result<CoreAction, ConfigError> {
		Ok(match self {
			Self::Filter(x) => CoreAction::Filter(x.parse()),
			Self::Transform(x) => CoreAction::Transform(x.parse()?),
		})
	}
}
