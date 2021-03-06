/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub(crate) struct File {
	path: PathBuf,
}

impl File {
	pub(crate) fn parse(self) -> source::File {
		source::File { path: self.path }
	}
}
