/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::Field;
use crate::error::FetcherConfigError as ConfigError;
use fetcher_core::action::{Action as CAction, filter::Contains as CContains};

type RegEx = String;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Contains(pub HashMap<Field, RegEx>);

impl Contains {
	pub fn parse(self) -> Result<Vec<CAction>, ConfigError> {
		self.0
			.into_iter()
			.map(|(field, re)| {
				Ok(CAction::Filter(Box::new(CContains::new(
					&re,
					field.parse(),
				)?)))
			})
			.collect()
	}
}
