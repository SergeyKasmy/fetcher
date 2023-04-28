/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source::email::ViewMode as CViewMode;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy, Default, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ViewMode {
	ReadOnly,
	#[default]
	MarkAsRead,
	Delete,
}

impl ViewMode {
	#[must_use]
	pub fn parse(self) -> CViewMode {
		use ViewMode::{Delete, MarkAsRead, ReadOnly};

		match self {
			ReadOnly => CViewMode::ReadOnly,
			MarkAsRead => CViewMode::MarkAsRead,
			Delete => CViewMode::Delete,
		}
	}
}
