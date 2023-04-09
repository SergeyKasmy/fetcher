/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod filter;

use self::filter::JobFilter;
use super::CONFIG_FILE_EXT;
use crate::{
	settings::{
		self, context::StaticContext as Context, external_data_provider::ExternalDataFromDataDir,
	},
	Jobs,
};
use fetcher_config::jobs::{
	named::{JobName, JobWithTaskNames},
	Job as ConfigJob,
};

use color_eyre::{eyre::eyre, Result};
use figment::{
	providers::{Format, Yaml},
	Figment,
};
use serde::Deserialize;
use std::{io, path::Path};
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

pub fn get_all(filter: Option<&[JobFilter]>, cx: Context) -> Result<Jobs> {
	cx.conf_paths
		.iter()
		.flat_map(|dir| get_all_from(dir, filter, cx))
		.collect()
}

pub fn get_all_from<'a>(
	cfg_dir: &'a Path,
	filter: Option<&'a [JobFilter]>,
	cx: Context,
) -> impl Iterator<Item = Result<(JobName, JobWithTaskNames)>> + 'a {
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
					match e.io_error().map(io::Error::kind) {
						Some(io::ErrorKind::NotFound) => (),
						_ => {
							tracing::warn!("File or directory is inaccessible: {e}");
						}
					}
					return None;
				}
			};

			let job_name: JobName = file
				.path()
				.strip_prefix(&jobs_dir)
				.expect("prefix should always be present because we just appended it")
				.with_extension("")
				.to_string_lossy()
				.into_owned()
				.into();

			// filter out all jobs that don't match the job filter
			if let Some(filter) = filter {
				if !filter.iter().any(|filter| filter.job_matches(&job_name)) {
					tracing::trace!("Filtering out job {job_name:?}");
					return None;
				}
			}

			let job = get(file.path(), job_name, cx)
				.map_err(|e| e.wrap_err(format!("invalid config at: {}", file.path().display())))
				.transpose()?;

			job.map(|(job_name, mut job)| {
				if let Some(filter) = filter {
					if let Some(task_name_map) = &job.task_names {
						job.inner.tasks = job
							.inner
							.tasks
							.into_iter()
							.enumerate()
							.filter_map(|(idx, task)| {
								let task_name = task_name_map.get(&idx).expect(
								"task name map should always contain all task indecies and names",
							);

								if filter
									.iter()
									.any(|filter| filter.task_matches(&job_name, task_name))
								{
									Some(task)
								} else {
									tracing::trace!(
										"Filtering out task {job_name:?}:{task_name:?}",
									);
									None
								}
							})
							.collect();

						if job.inner.tasks.is_empty() {
							// TODO: list task filter and all available tasks (from task_name_map)
							tracing::warn!(
								"Asked to run job {job_name} but no tasks matched the task filter"
							);
							return None;
						}
					}
				}

				Some((job_name, job))
			})
			.transpose()
		})
}

#[tracing::instrument(skip(cx))]
pub fn get(path: &Path, name: JobName, cx: Context) -> Result<Option<(JobName, JobWithTaskNames)>> {
	tracing::trace!("Parsing a job from file");

	// TODO: use extract_inner()
	let TemplatesField { templates } = Figment::new().merge(Yaml::file(path)).extract()?;

	let mut full_conf = Figment::new();

	// prepend templates
	if let Some(templates) = templates {
		for tmpl_name in templates {
			let tmpl = settings::config::templates::find(&tmpl_name, cx)?
				.ok_or_else(|| eyre!("Template \"{tmpl_name}\" not found"))?;

			tracing::trace!("Using template: {:?} at {:?}", tmpl.name, tmpl.path);

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
