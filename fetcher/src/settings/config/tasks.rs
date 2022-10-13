/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add trace logging, e.g. all config dirs, all config files, stuff like that

use super::CONFIG_FILE_EXT;
use crate::settings::{
	self, context::StaticContext as Context, external_data::ExternalDataFromDataDir,
};
use fetcher_config::tasks::{task::Task as ConfigTask, ParsedTask, ParsedTasks};

use color_eyre::eyre::eyre;
use color_eyre::Result;
use figment::{
	providers::{Format, Yaml},
	Figment,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

const TASKS_DIR_NAME: &str = "tasks";

#[derive(Deserialize, Debug)]
struct DisabledField {
	disabled: fetcher_config::tasks::task::DisabledField,
}

#[derive(Deserialize, Debug)]
struct TemplatesField {
	templates: fetcher_config::tasks::task::TemplatesField,
}

// #[tracing::instrument(name = "settings:task", skip(settings))]
#[tracing::instrument(skip(cx))]
pub fn get_all(cx: Context) -> Result<ParsedTasks> {
	let mut tasks = ParsedTasks::new();
	for dir in &cx.conf_paths {
		get_all_from(dir, &mut tasks, cx)?;
	}

	Ok(tasks)
}

pub fn get_all_from(cfg_dir: &Path, out_tasks: &mut ParsedTasks, cx: Context) -> Result<()> {
	let glob_str = format!(
		"{cfg_dir}/{TASKS_DIR_NAME}/**/*.{CONFIG_FILE_EXT}",
		cfg_dir = cfg_dir.to_str().expect("Path is illegal UTF-8") // FIXME
	);

	let cfgs = glob::glob(&glob_str)
		.expect("The glob pattern is hand-made and should never fail to be parsed");

	for cfg in cfgs {
		let cfg = cfg?;
		let name = cfg
			.strip_prefix(cfg_dir)
			.expect("shouldn't fail since cfg_dir has just been prepended")
			.strip_prefix(TASKS_DIR_NAME)
			.expect("shouldn't fail since TASKS_DIR_NAME has just been prepended")
			.with_extension("")
			.to_string_lossy()
			.into_owned();

		get(cfg, &name, cx)?.map(|task| out_tasks.insert(name, task));
	}

	Ok(())
}

#[tracing::instrument(skip(cx))]
pub fn get(path: PathBuf, name: &str, cx: Context) -> Result<Option<ParsedTask>> {
	tracing::trace!("Parsing a task from file");

	let task_file = Figment::new().merge(Yaml::file(&path));

	let DisabledField { disabled } = task_file.extract()?;

	if disabled.unwrap_or(false) {
		tracing::trace!("Task is disabled, skipping...");
		return Ok(None);
	}

	let TemplatesField { templates } = task_file.extract()?;

	let mut full_conf = Figment::new();

	if let Some(templates) = templates {
		for tmpl_name in templates {
			let tmpl = settings::config::templates::find(&tmpl_name, cx)?
				.ok_or_else(|| eyre!("Template not found"))?;

			tracing::trace!("Using template: {:?}", tmpl.path);

			full_conf = full_conf.merge(Yaml::string(&tmpl.contents));
		}
	}

	let full_conf = full_conf.merge(Yaml::file(&path));
	let task: ConfigTask = full_conf.extract()?;

	Ok(Some(task.parse(name, &ExternalDataFromDataDir { cx })?))
}
