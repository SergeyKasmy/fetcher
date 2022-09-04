/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::CONFIG_FILE_EXT;
use fetcher_config::error::ConfigError;
use fetcher_core::task::template::Template;

use std::fs;
use std::path::Path;

const TEMPLATES_DIR: &str = "templates";

#[tracing::instrument(name = "template")]
pub fn find(name: &str) -> Result<Option<Template>, ConfigError> {
	for template_dir_path in super::cfg_dirs()?.into_iter().map(|mut p| {
		p.push(TEMPLATES_DIR);
		p
	}) {
		if let Some(template) = find_in(&template_dir_path, name)? {
			return Ok(Some(template));
		}
	}

	Ok(None)
}

pub fn find_in(templates_path: &Path, name: &str) -> Result<Option<Template>, ConfigError> {
	tracing::trace!("Searching for template in {}", templates_path.display());
	let path = templates_path.join(name).with_extension(CONFIG_FILE_EXT);
	if !path.is_file() {
		tracing::trace!("{path:?} is not a file");
		// return Err(Error::TemplateNotFound(name));
		return Ok(None);
	}

	let contents = fs::read_to_string(&path).map_err(|e| ConfigError::Read(e, path.clone()))?;

	Ok(Some(Template {
		name: name.to_owned(),
		path,
		contents,
	}))
}
