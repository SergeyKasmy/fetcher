/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::str::FromStr;

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
	ReadOnly,
	MarkAsRead,
	Delete,
}

impl FromStr for ViewMode {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		Ok(match s {
			"read_only" => Self::ReadOnly,
			"mark_as_read" => Self::MarkAsRead,
			"delete" => Self::Delete,
			_ => {
				return Err(Error::ConfigInvalidFieldType {
					name: "Email".to_string(),
					field: "view_mode",
					expected_type: "string (read_only | mark_as_read | delete)",
				})
			}
		})
	}
}
