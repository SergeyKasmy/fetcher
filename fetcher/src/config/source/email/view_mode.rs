/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub(crate) enum ViewMode {
	ReadOnly,
	MarkAsRead,
	Delete,
}

impl ViewMode {
	pub(crate) fn parse(self) -> source::email::ViewMode {
		use ViewMode::{Delete, MarkAsRead, ReadOnly};

		match self {
			ReadOnly => source::email::ViewMode::ReadOnly,
			MarkAsRead => source::email::ViewMode::MarkAsRead,
			Delete => source::email::ViewMode::Delete,
		}
	}
}
