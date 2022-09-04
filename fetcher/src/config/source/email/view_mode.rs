/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
	ReadOnly,
	MarkAsRead,
	Delete,
}

impl ViewMode {
	pub fn parse(self) -> source::with_custom_rf::email::ViewMode {
		use ViewMode::{Delete, MarkAsRead, ReadOnly};

		match self {
			ReadOnly => source::with_custom_rf::email::ViewMode::ReadOnly,
			MarkAsRead => source::with_custom_rf::email::ViewMode::MarkAsRead,
			Delete => source::with_custom_rf::email::ViewMode::Delete,
		}
	}
}
