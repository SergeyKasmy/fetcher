/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: use directories instead of xdg

use std::{
	fs,
	path::{Path, PathBuf},
};

use super::PREFIX;
use crate::error::{Error, Result};

const CONFIG_FILE_EXT: &str = ".yaml";

/// Find all task .toml configs in the first non-empty config directory by priority
/// Ignore configs in directories lower in priority if one in higher priority has configs    // TODO: isn't just ignoring them kinda stupid?
/// Returns (file contents, file path)
pub fn tasks() -> Result<Vec<(String, PathBuf)>> {
	let cfg_dirs = if cfg!(debug_assertions) {
		// TODO: get that dir from env var
		vec![PathBuf::from("debug_data/cfg".to_string())]
	} else {
		let base_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;

		let mut cfg_dirs = Vec::with_capacity(2);
		cfg_dirs.push(base_dirs.get_config_home());
		cfg_dirs.append(&mut base_dirs.get_config_dirs());

		cfg_dirs
	};

	let mut cfgs = Vec::new();
	// TODO: add trace logging, e.g. all config dirs, all config files, stuff like that
	for cfg_dir in cfg_dirs {
		if !cfgs.is_empty() {
			// stop if we have already founds configs in the previous dir
			break;
		}

		// find all configs in the current path
		let cfg_files = glob::glob(&format!(
			"{cfg_dir}/tasks/**/*{CONFIG_FILE_EXT}",
			cfg_dir = cfg_dir
				.to_str()
				.expect("Non unicode paths are currently unsupported")
		))
		.unwrap(); // unwrap NOTE: should be safe if the glob pattern is correct

		for cfg_file in cfg_files {
			match cfg_file {
				Ok(cfg_file) => cfgs.push(cfg_file),
				// just log the error here because there may be other valid config files in the directory
				Err(e) => tracing::warn!(
					"Config {path} is inaccessable: {err}",
					path = e.path().display(),
					err = e.error(),
				),
			}
		}
	}

	cfgs.into_iter()
		.map(|path| {
			Ok((
				// NOTE: sadly can't re-order them to be in the more natural (path, contents)
				// because read_to_string() borrows path and path would've been already moved to the tuple...
				fs::read_to_string(&path)
					.map_err(|e| Error::InaccessibleConfig(e, path.clone()))?,
				path,
			))
		})
		.collect()
}

pub fn template(name: &Path) -> Result<(String, PathBuf)> {
	let cfg_dirs = if cfg!(debug_assertions) {
		vec![PathBuf::from("debug_data/cfg".to_string())]
	} else {
		let base_dirs = xdg::BaseDirectories::with_prefix(PREFIX)?;

		let mut cfg_dirs = Vec::with_capacity(2);
		cfg_dirs.push(base_dirs.get_config_home());
		cfg_dirs.append(&mut base_dirs.get_config_dirs());

		cfg_dirs
	}
	.into_iter()
	.map(|mut p| {
		p.push("templates");
		p
	});

	let tmpl_path = cfg_dirs.into_iter().find_map(|mut dir| {
		let mut file_name = name.as_os_str().to_owned();
		file_name.push(CONFIG_FILE_EXT);

		dir.push(file_name);
		if dir.exists() {
			Some(dir)
		} else {
			None
		}
	});

	let (template, tmpl_path) = match tmpl_path {
		Some(tmpl_path) => {
			let s = fs::read_to_string(&tmpl_path)
				.map_err(|e| Error::InaccessibleConfig(e, tmpl_path.clone()))?;

			(s, tmpl_path)
		}
		None => return Err(Error::TemplateNotFound(name.to_owned())),
	};

	Ok((template, tmpl_path))
}
