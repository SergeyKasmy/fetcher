/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::CONFIG_FILE_EXT;
use crate::settings::context::StaticContext as Context;
use fetcher_config::jobs::action::Action as ActionConfig;

use color_eyre::{Result, eyre::eyre};
use figment::{
	Figment,
	providers::{Format, Yaml},
};
use std::path::Path;

const ACTIONS_DIR: &str = "actions";

/// Find all actions with `name` in the default actions paths
///
/// # Errors
/// if the found actions path couldn't be read
#[tracing::instrument(level = "debug", name = "action")]
pub fn find(name: &str, context: Context) -> Result<Option<Vec<ActionConfig>>> {
	for actions_dir_path in context.conf_paths.iter().map(|p| p.join(ACTIONS_DIR)) {
		if let Some(actions) = find_in(&actions_dir_path, name)? {
			return Ok(Some(actions));
		}
	}

	Ok(None)
}

/// Find all actions with `name` in `actions_path`.
/// Returns Some(ActionConfig) if the action was found in the directory, None otherwise
///
/// # Errors
/// * if the path couldn't be read
/// * if the config exists at `action_path` but is invalid
pub fn find_in(action_path: &Path, name: &str) -> Result<Option<Vec<ActionConfig>>> {
	tracing::trace!("Searching for an action {name:?} in {action_path:?}");
	let path = action_path.join(name).with_extension(CONFIG_FILE_EXT);

	if !path.exists() {
		tracing::trace!("{path:?} doesn't exist");
		return Ok(None);
	}

	// TODO: replace with .is_dir() because .is_file() doesn't cover unix special file types and windows symlinks
	if !path.is_file() {
		return Err(eyre!(
			"Action \"{name}\" exists at {} but is not a file",
			path.display()
		));
	}

	let action_config: Vec<ActionConfig> = Figment::new().merge(Yaml::file(path)).extract()?;

	Ok(Some(action_config))
}
