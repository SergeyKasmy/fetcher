/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::CONFIG_FILE_EXT;
use crate::settings::context::StaticContext as Context;

use color_eyre::Result;
use std::fs;
use std::path::{Path, PathBuf};

const TEMPLATES_DIR: &str = "templates";

#[derive(Debug)]
pub struct Template {
	pub name: String,
	pub path: PathBuf,
	pub contents: String,
}

#[tracing::instrument(name = "template")]
pub fn find(name: &str, context: Context) -> Result<Option<Template>> {
	for template_dir_path in context.conf_paths.iter().map(|p| p.join(TEMPLATES_DIR)) {
		if let Some(template) = find_in(&template_dir_path, name)? {
			return Ok(Some(template));
		}
	}

	Ok(None)
}

pub fn find_in(templates_path: &Path, name: &str) -> Result<Option<Template>> {
	tracing::debug!("Searching for a template {name:?} in {templates_path:?}");
	let path = templates_path.join(name).with_extension(CONFIG_FILE_EXT);
	if !path.is_file() {
		tracing::trace!("{path:?} is not a file");
		return Ok(None);
	}

	let contents = fs::read_to_string(&path)?;

	Ok(Some(Template {
		name: name.to_owned(),
		path,
		contents,
	}))
}
