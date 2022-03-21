/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: add trace logging, e.g. all config dirs, all config files, stuff like that

use fetcher::{
	config::{self, DataSettings, TemplatesField},
	error::{Error, Result},
	task::{NamedTask, Tasks},
};
use figment::{
	providers::{Format, Yaml},
	Figment,
};
use itertools::Itertools; // for .flatten_ok()
use std::path::{Path, PathBuf};

use super::CONFIG_FILE_EXT;
use crate::settings;

// #[tracing::instrument(name = "settings:task", skip(settings))]
#[tracing::instrument(skip(settings))]
pub fn get_all(settings: &DataSettings) -> Result<Tasks> {
	super::cfg_dirs()?
		.into_iter()
		.map(|mut p| {
			p.push("tasks");
			p
		})
		.map(|d| get_all_from(d, settings))
		.flatten_ok()
		.collect()
}

pub fn get_all_from(tasks_dir: PathBuf, settings: &DataSettings) -> Result<Tasks> {
	let glob_str = format!(
		"{tasks_dir}/**/*.{CONFIG_FILE_EXT}",
		tasks_dir = tasks_dir
			.to_str()
			.ok_or_else(|| Error::BadPath(tasks_dir.clone()))?
	);

	let cfgs = glob::glob(&glob_str).unwrap(); // unwrap NOTE: should be safe if the glob pattern is correct

	cfgs.into_iter()
		.filter_map(|c| match c {
			Ok(v) => get(v, settings).transpose(), // TODO: is that okay?
			Err(e) => Some(Err(Error::LocalIoRead(e.into_error(), tasks_dir.clone()))),
		})
		.collect()
}

#[tracing::instrument(skip(settings))]
pub fn get(path: PathBuf, settings: &DataSettings) -> Result<Option<NamedTask>> {
	tracing::trace!("Parsing a task from file");
	fn name(path: &Path) -> Option<String> {
		Some(path.file_stem()?.to_str()?.to_owned())
	}

	let templates: TemplatesField = Figment::new()
		.merge(Yaml::file(&path))
		.extract()
		.map_err(|e| Error::InvalidConfigFormat(e, path.clone()))?;

	let mut conf = Figment::new();

	if let Some(templates) = templates.templates {
		for tmpl_name in templates {
			let tmpl = settings::config::templates::find(&tmpl_name)?
				.ok_or_else(|| Error::TemplateNotFound(tmpl_name.clone(), path.clone()))?;

			tracing::trace!("Using template: {:?}", tmpl.path);

			conf = conf.merge(Yaml::string(&tmpl.contents));
		}
	}

	let task: config::Task = conf
		.merge(Yaml::file(&path))
		.extract()
		.map_err(|e| Error::InvalidConfigFormat(e, path.clone()))?;

	let task = task.parse(&path, settings)?;
	if task.disabled {
		tracing::trace!("Task is disabled, skipping...");
		return Ok(None);
	}

	Ok(Some(task.into_named_task(
		name(&path).ok_or_else(|| Error::BadPath(path.clone()))?,
		path,
	)))
}
