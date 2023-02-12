/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add trace logging, e.g. all config dirs, all config files, stuff like that

use super::CONFIG_FILE_EXT;
use crate::{
	settings::{self, context::StaticContext as Context, external_data::ExternalDataFromDataDir},
	Jobs,
};
use fetcher_config::jobs::Job as ConfigJob;

use color_eyre::{eyre::eyre, Result};
use fetcher_core::job::Job;
use figment::{
	providers::{Format, Yaml},
	Figment,
};
use serde::Deserialize;
use std::path::Path;
use walkdir::WalkDir;

const TASKS_DIR_NAME: &str = "tasks";

#[derive(Deserialize, Debug)]
struct DisabledField {
	disabled: fetcher_config::jobs::job::DisabledField,
}

#[derive(Deserialize, Debug)]
struct TemplatesField {
	templates: fetcher_config::jobs::job::TemplatesField,
}

#[tracing::instrument(skip(cx))]
pub fn get_all(by_name: Option<&[&str]>, cx: Context) -> Result<Jobs> {
	cx.conf_paths
		.iter()
		.flat_map(|dir| get_all_from(dir, by_name, cx))
		.collect()
}

pub fn get_all_from<'a>(
	cfg_dir: &'a Path,
	by_name: Option<&'a [&'a str]>,
	cx: Context,
) -> impl Iterator<Item = Result<(String, Job)>> + 'a {
	WalkDir::new(cfg_dir.join(TASKS_DIR_NAME))
		.follow_links(true)
		.into_iter()
		.filter_map(move |cfg| {
			let cfg = match cfg {
				Ok(dir_entry) => {
					// filter out files with no extension
					let Some(ext) = dir_entry.path().extension() else {
						return None;
					};

					// or if the extension isn't CONFIG_FILE_EXT
					if ext != CONFIG_FILE_EXT {
						return None;
					}

					dir_entry
				}
				Err(e) => {
					tracing::warn!("File or directory is inaccessible: {e}");
					return None;
				}
			};

			let file_name = Path::new(cfg.file_name());
			let task_name = file_name.with_extension("").to_string_lossy().into_owned();

			// if asked to find only tasks with names `by_name`,
			// check if the current task name is in the list and filter it out if not
			if let Some(by_name) = by_name {
				if !by_name.iter().any(|x| *x == task_name) {
					return None;
				}
			}

			let task = get(cfg.path(), &task_name, cx).transpose()?;
			let named_task = task.map(|t| (task_name, t));

			Some(named_task)
		})
}

#[tracing::instrument(skip(cx))]
pub fn get(path: &Path, name: &str, cx: Context) -> Result<Option<Job>> {
	tracing::trace!("Parsing a task from file");

	let task_file = Figment::new().merge(Yaml::file(path));

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

	let full_conf = full_conf.merge(Yaml::file(path));
	let task: ConfigJob = full_conf.extract()?;

	Ok(Some(task.parse(name, &ExternalDataFromDataDir { cx })?))
}
