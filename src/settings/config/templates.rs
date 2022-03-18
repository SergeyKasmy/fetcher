/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::fs;
use std::path::PathBuf;

use fetcher::error::Error;
use fetcher::task::template::Template;

use super::CONFIG_FILE_EXT;

pub fn find(name: String) -> Result<Template> {
	Ok(super::cfg_dirs()?
		.into_iter()
		.map(|mut p| {
			p.push("templates");
			p
		})
		.find_map(|p| find_in(p, name)?))
}

pub fn find_in(mut templates_path: PathBuf, name: String) -> Result<Option<Template>> {
	let path = templates_path.join(name).with_extension(CONFIG_FILE_EXT);
	if !path.is_file() {
		// return Err(Error::TemplateNotFound(name));
		return Ok(None);
	}

	let contents =
		fs::read_to_string(&path).map_err(|e| Error::InaccessibleConfig(e, path.clone()))?;

	Ok(Template {
		name,
		path,
		contents,
	})
}
