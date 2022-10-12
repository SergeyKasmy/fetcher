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
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct DisabledField {
	disabled: fetcher_config::tasks::task::DisabledField,
}

#[derive(Deserialize, Debug)]
struct TemplatesField {
	templates: fetcher_config::tasks::task::TemplatesField,
}

// #[tracing::instrument(name = "settings:task", skip(settings))]
#[tracing::instrument]
pub async fn get_all(context: Context) -> Result<ParsedTasks> {
	let mut tasks = ParsedTasks::new();
	for dir in context.conf_paths.iter().map(|p| p.join("tasks")) {
		// TODO: make a stream?
		tasks.extend(get_all_from(dir, context).await?);
	}

	Ok(tasks)
}

pub async fn get_all_from(tasks_dir: PathBuf, context: Context) -> Result<ParsedTasks> {
	let glob_str = format!(
		"{tasks_dir}/**/*.{CONFIG_FILE_EXT}",
		tasks_dir = tasks_dir.to_str().expect("Path is illegal UTF-8")
	);

	let cfgs = glob::glob(&glob_str)
		.expect("The glob pattern is hand-made and should never fail to be parsed");

	let mut tasks = ParsedTasks::new();
	for cfg in cfgs {
		let cfg = cfg?;
		let name = cfg
			.strip_prefix(&tasks_dir)
			.expect("The prefix was just appended up above in the glob pattern and thus should never fail")
			.with_extension("")
			.to_string_lossy()	// TODO: choose if we should use lossy or fail on invalid UTF-8 like in read_filter::get. This inconsistent behavior is probably even worse than any of the two
			.into_owned();

		get(cfg, &name, context)
			.await?
			.map(|task| tasks.insert(name, task));
	}

	Ok(tasks)
}

#[tracing::instrument]
pub async fn get(path: PathBuf, name: &str, context: Context) -> Result<Option<ParsedTask>> {
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
			let tmpl = settings::config::templates::find(&tmpl_name, context)?
				.ok_or_else(|| eyre!("Template not found"))?;

			tracing::trace!("Using template: {:?}", tmpl.path);

			full_conf = full_conf.merge(Yaml::string(&tmpl.contents));
		}
	}

	let full_conf = full_conf.merge(Yaml::file(&path));
	let task: ConfigTask = full_conf.extract()?;

	let name = name.to_owned(); // ehhhh, such a wasteful clone, and just because tokio doesn't support scoped tasks
	let parsed_task = tokio::task::spawn_blocking(move || {
		task.parse(&name, &ExternalDataFromDataDir { cx: context })
	})
	.await
	.unwrap()?;
	// let parsed_task = task.parse(name, &settings::TaskSettingsFetcherDefault)?;

	Ok(Some(parsed_task))
}
