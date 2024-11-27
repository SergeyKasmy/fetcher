/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use serde_with::{OneOrMany, serde_as};

#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct StringSource(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<String>);

impl StringSource {
	#[must_use]
	pub fn parse(self) -> Vec<String> {
		self.0.into_iter().collect()
	}
}
