/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod config;
pub mod context;
pub mod data;
pub mod external_data_provider;
pub mod log;

use color_eyre::{eyre::eyre, Result};
use directories::ProjectDirs;
use once_cell::sync::OnceCell;

const PREFIX: &str = "fetcher";

pub fn proj_dirs() -> Result<&'static ProjectDirs> {
	static PROJ_DIRS: OnceCell<ProjectDirs> = OnceCell::new();

	PROJ_DIRS.get_or_try_init(|| {
		ProjectDirs::from("", "", PREFIX).ok_or_else(|| {
			eyre!(
				"No valid default project directories found. Specify manually via launch arguments"
			)
		})
	})
}
