/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::CONFIG_FILE_EXT;
use crate::settings::context::StaticContext as Context;

use color_eyre::{Result, eyre::eyre};
use std::fs;
use std::path::{Path, PathBuf};

const TEMPLATES_DIR: &str = "templates";

#[derive(Debug)]
pub struct Template {
	pub name: String,
	pub path: PathBuf,
	pub contents: String,
}

/// Find all templates with `name` in the default templates paths
///
/// # Errors
/// if the found template path couldn't be read
#[tracing::instrument(level = "debug", name = "template")]
pub fn find(name: &str, context: Context) -> Result<Option<Template>> {
	for template_dir_path in context.conf_paths.iter().map(|p| p.join(TEMPLATES_DIR)) {
		if let Some(template) = find_in(&template_dir_path, name)? {
			return Ok(Some(template));
		}
	}

	Ok(None)
}

/// Find all templates with `name` in `templates_path`.
/// Returns Some(Template) if the template was found in the directory, None otherwise
///
/// # Errors
/// if the path couldn't be read
pub fn find_in(templates_path: &Path, name: &str) -> Result<Option<Template>> {
	tracing::trace!("Searching for a template {name:?} in {templates_path:?}");
	let path = templates_path.join(name).with_extension(CONFIG_FILE_EXT);

	if !path.exists() {
		tracing::trace!("{path:?} doesn't exist");
		return Ok(None);
	}

	// TODO: replace with .is_dir() because .is_file() doesn't cover unix special file types and windows symlinks
	if !path.is_file() {
		// tracing::trace!("{path:?} exists but is not a file");
		return Err(eyre!(
			"Template \"{name}\" exists at {} but is not a file",
			path.display()
		));
	}

	let contents = fs::read_to_string(&path)?;

	Ok(Some(Template {
		name: name.to_owned(),
		path,
		contents,
	}))
}
