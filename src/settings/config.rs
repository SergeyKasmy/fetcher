/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub mod tasks;
pub mod templates;

use fetcher::error::Result;
use std::path::PathBuf;

const CONFIG_FILE_EXT: &str = ".yaml";

// TODO: use directories instead of xdg
fn cfg_dirs() -> Result<Vec<PathBuf>> {
	Ok(if cfg!(debug_assertions) {
		vec![PathBuf::from("debug_data/cfg".to_string())]
	} else {
		let base_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;

		let mut dirs = Vec::with_capacity(2);
		dirs.push(base_dirs.get_config_home());
		dirs.append(&mut base_dirs.get_config_dirs());

		dirs
	})
}
