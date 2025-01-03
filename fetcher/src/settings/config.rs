/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod actions;
pub mod jobs;
pub mod templates;

#[cfg_attr(not(target_os = "linux"), expect(unused_imports))]
use super::PREFIX;
use super::proj_dirs;

use color_eyre::Result;
use std::path::PathBuf;

const CONFIG_FILE_EXT: &str = "yml";

pub fn default_cfg_dirs() -> Result<Vec<PathBuf>> {
	#[allow(unused_mut)] // requred to be mutable only on linux
	let mut dirs = vec![proj_dirs()?.config_dir().to_path_buf()];

	#[cfg(target_os = "linux")]
	{
		dirs.push(format!("/etc/xdg/{PREFIX}").into());
	}

	Ok(dirs)
}
