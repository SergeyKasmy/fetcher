/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct File {
	path: PathBuf,
}

impl File {
	pub fn parse(self) -> source::File {
		source::File { path: self.path }
	}
}
