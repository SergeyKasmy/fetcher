/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source::File as CFile;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};
use std::path::PathBuf;

#[serde_as]
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(transparent)]
pub struct File(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<PathBuf>);

impl File {
	#[must_use]
	pub fn parse(self) -> Vec<CFile> {
		self.0.into_iter().map(|path| CFile { path }).collect()
	}
}
