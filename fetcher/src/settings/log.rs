/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::proj_dirs;
use crate::extentions::ErrorChainExt;
use fetcher_core::action::transform::error::TransformError;

use color_eyre::Result;
use std::{
	fs,
	path::PathBuf,
	time::{SystemTime, UNIX_EPOCH},
};

/// Use `$XDG_STATE_HOME` or OS native cache dir for logs if run as normal user or `/var/log/fetcher` if run as root on linux
pub fn default_log_path() -> Result<PathBuf> {
	#[cfg(target_os = "linux")]
	{
		if nix::unistd::Uid::effective().is_root() {
			return Ok(PathBuf::from("/var/log/fetcher"));
		}
	}

	let proj = proj_dirs()?;

	// use $XDG_STATE_HOME on linux and data dir on other platforms
	Ok(proj
		.state_dir()
		.unwrap_or_else(|| proj.cache_dir())
		.to_path_buf())
}

#[allow(rustdoc::invalid_html_tags)]
/// Save entry contents from a task to a <task name>/<current time>.txt file in `default_log_path()` dir
pub fn log_transform_err(e: &TransformError, job_name: &str) -> Result<()> {
	let root_dir = default_log_path()?;
	let err_id = format!(
		"errors/{job_name}/{}",
		SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
	);
	let log_dir = root_dir.join(err_id);

	fs::create_dir_all(&log_dir)?;
	fs::write(log_dir.join("error.txt"), e.display_chain())?;
	fs::write(
		log_dir.join("entry.txt"),
		format!("{:#?}", e.original_entry),
	)?;

	if let Some(raw_contents) = e.original_entry.raw_contents.as_ref() {
		fs::write(log_dir.join("raw_contents.txt"), raw_contents)?;
	}

	Ok(())
}
