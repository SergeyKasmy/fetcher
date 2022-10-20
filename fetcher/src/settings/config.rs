/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod tasks;
pub mod templates;

use super::PREFIX;

use std::{io, path::PathBuf};

const CONFIG_FILE_EXT: &str = "yml";

// TODO: use directories instead of xdg
pub fn default_cfg_dirs() -> io::Result<Vec<PathBuf>> {
	let base_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;

	let mut dirs = Vec::with_capacity(2);
	dirs.push(base_dirs.get_config_home());
	dirs.append(&mut base_dirs.get_config_dirs());

	Ok(dirs)
}
