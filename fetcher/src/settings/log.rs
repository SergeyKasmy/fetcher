/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::PREFIX;
use fetcher_core::error::{transform::Error as TransformError, ErrorChainExt};

use color_eyre::Result;
use std::{
	io,
	path::PathBuf,
	time::{SystemTime, UNIX_EPOCH},
};
use tokio::fs;

/// Use `$XDG_STATE_HOME/fetcher/log` dir for logs if run as normal user or `/var/log/fetcher` if run as root
pub fn default_log_path() -> io::Result<PathBuf> {
	if nix::unistd::Uid::effective().is_root() {
		Ok(PathBuf::from("/var/log/fetcher"))
	} else {
		Ok(xdg::BaseDirectories::with_prefix(PREFIX)?
			.get_state_home()
			.join("log"))
	}
}

/// Save entry contents from a task to a <task name>/<current time>.txt file in `default_log_path()` dir
pub async fn log_transform_err(e: &TransformError, task_name: &str) -> Result<()> {
	let root_dir = default_log_path()?;
	let err_id = format!(
		"errors/{task_name}/{}",
		SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
	);
	let log_dir = root_dir.join(err_id);

	fs::create_dir_all(&log_dir).await?;
	fs::write(log_dir.join("error.txt"), e.display_chain()).await?;
	fs::write(
		log_dir.join("entry.txt"),
		format!("{:#?}", e.original_entry),
	)
	.await?;

	if let Some(raw_contents) = e.original_entry.raw_contents.as_ref() {
		fs::write(log_dir.join("raw_contents.txt"), raw_contents).await?;
	}

	Ok(())
}
