/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use fetcher_core::source::email::ViewMode as CViewMode;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
	ReadOnly,
	MarkAsRead,
	Delete,
}

impl ViewMode {
	pub fn parse(self) -> CViewMode {
		use ViewMode::{Delete, MarkAsRead, ReadOnly};

		match self {
			ReadOnly => CViewMode::ReadOnly,
			MarkAsRead => CViewMode::MarkAsRead,
			Delete => CViewMode::Delete,
		}
	}
}
