/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source::Exec as CExec;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

#[serde_as]
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(transparent)]
pub struct Exec {
	#[serde_as(deserialize_as = "OneOrMany<_>")]
	pub cmd: Vec<String>,
}

impl Exec {
	pub fn parse(self) -> Vec<CExec> {
		self.cmd.into_iter().map(|cmd| CExec { cmd }).collect()
	}
}
