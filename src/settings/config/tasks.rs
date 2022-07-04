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
	task::{Task, Tasks},
};
use figment::{
	providers::{Format, Yaml},
	Figment,
};
use std::path::PathBuf;

use super::CONFIG_FILE_EXT;
use crate::settings;

// #[tracing::instrument(name = "settings:task", skip(settings))]
#[tracing::instrument(skip(settings))]
pub async fn get_all(settings: &DataSettings) -> Result<Tasks> {
	let mut tasks = Tasks::new();
	for dir in super::cfg_dirs()?.into_iter().map(|mut p| {
		p.push("tasks");
		p
	}) {
		tasks.extend(get_all_from(dir, settings).await?);
	}

	Ok(tasks)
}

pub async fn get_all_from(tasks_dir: PathBuf, settings: &DataSettings) -> Result<Tasks> {
	let glob_str = format!(
		"{tasks_dir}/**/*.{CONFIG_FILE_EXT}",
		tasks_dir = tasks_dir
			.to_str()
			.ok_or_else(|| Error::BadPath(tasks_dir.clone()))?
	);

	let cfgs = glob::glob(&glob_str).unwrap(); // unwrap NOTE: should be safe if the glob pattern is correct

	let mut tasks = Tasks::new();
	for cfg in cfgs {
		let cfg = cfg.map_err(|e| Error::LocalIoRead(e.into_error(), tasks_dir.clone()))?;
		let name = cfg
			.strip_prefix(&tasks_dir)
			.unwrap()
			.with_extension("")
			.to_string_lossy()
			.into_owned();

		get(cfg, &name, settings)
			.await?
			.map(|task| tasks.insert(name, task));
	}

	Ok(tasks)
}

#[tracing::instrument(skip(settings))]
pub async fn get(path: PathBuf, name: &str, settings: &DataSettings) -> Result<Option<Task>> {
	tracing::trace!("Parsing a task from file");

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

	let task = task.parse(name, settings).await?;

	// TODO: move that check up above and skip all parsing if the task's disabled to avoid terminating if a disabled task's config is corrupted
	if task.disabled {
		tracing::trace!("Task is disabled, skipping...");
		return Ok(None);
	}

	Ok(Some(task))
}
