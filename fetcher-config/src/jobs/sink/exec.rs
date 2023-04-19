/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::sink::Exec as CExec;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, PartialEq, Default, Debug)]
#[serde(transparent)]
pub struct Exec {
	pub cmd: String,
}

impl Exec {
	pub fn parse(self) -> CExec {
		CExec { cmd: self.cmd }
	}
}
