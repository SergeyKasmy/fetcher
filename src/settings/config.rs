/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::{fs, path::PathBuf};

use super::PREFIX;
use crate::error::{Error, Result};

const CONFIG: &str = "config.toml";

pub fn config() -> Result<String> {
	let path = if !cfg!(debug_assertions) {
		xdg::BaseDirectories::with_prefix(PREFIX)?
			.place_config_file(CONFIG)
			.map_err(Error::InaccessibleConfig)?
	} else {
		PathBuf::from(format!("debug_data/{CONFIG}"))
	};

	fs::read_to_string(&path).map_err(Error::InaccessibleConfig)
}
