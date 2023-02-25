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

const JOBS_DIR_NAME: &str = "jobs";

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
	let jobs_dir = cfg_dir.join(JOBS_DIR_NAME);
	tracing::trace!("Searching for job configs in {jobs_dir:?}");

	WalkDir::new(&jobs_dir)
		.follow_links(true)
		.into_iter()
		.filter_map(move |dir_entry| {
			let file = match dir_entry {
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

			let job_name = file
				.path()
				.strip_prefix(&jobs_dir)
				.expect("prefix should always be present because we just appended it")
				.with_extension("")
				.to_string_lossy()
				.into_owned();

			// if asked to find only tasks with names `by_name`,
			// check if the current task name is in the list and filter it out if not
			if let Some(by_name) = by_name {
				if !by_name.iter().any(|x| *x == job_name) {
					return None;
				}
			}

			let task = get(file.path(), &job_name, cx)
				.map_err(|e| e.wrap_err(format!("Invalid config at: {}", file.path().display())))
				.transpose()?;
			let named_task = task.map(|t| (job_name, t));

			Some(named_task)
		})
}

#[tracing::instrument(skip(cx))]
pub fn get(path: &Path, name: &str, cx: Context) -> Result<Option<Job>> {
	tracing::trace!("Parsing a task from file");

	let TemplatesField { templates } = Figment::new().merge(Yaml::file(path)).extract()?;

	let mut full_conf = Figment::new();

	// prepend templates
	if let Some(templates) = templates {
		for tmpl_name in templates {
			let tmpl = settings::config::templates::find(&tmpl_name, cx)?
				.ok_or_else(|| eyre!("Template not found"))?;

			tracing::trace!("Using template: {:?}", tmpl.path);

			full_conf = full_conf.merge(Yaml::string(&tmpl.contents));
		}
	}

	// append the config itself
	let full_conf = full_conf.merge(Yaml::file(path));

	// extract the disabled field and ignore the config if it's set to true
	let DisabledField { disabled } = full_conf.extract()?;
	if disabled.unwrap_or(false) {
		tracing::trace!("Job is disabled, skipping...");
		return Ok(None);
	}

	let job: ConfigJob = full_conf.extract()?;

	Ok(Some(job.parse(name, &ExternalDataFromDataDir { cx })?))
}
