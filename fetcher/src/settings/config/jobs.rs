/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod filter;

use self::filter::JobFilter;
use super::CONFIG_FILE_EXT;
use crate::{
	Jobs,
	settings::{
		self, context::StaticContext as Context, external_data_provider::ExternalDataFromDataDir,
	},
};
use fetcher_config::jobs::{
	Job as ConfigJob,
	named::{JobName, JobWithTaskNames},
};

use color_eyre::{Result, eyre::eyre};
use figment::{
	Figment,
	providers::{Format, Yaml},
};
use serde::Deserialize;
use std::{fmt::Write, io, path::Path};
use walkdir::{DirEntry, WalkDir};

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

pub fn get_all_from(
	cfg_dir: &Path,
	filter: Option<&[JobFilter]>,
	cx: Context,
) -> impl Iterator<Item = Result<(JobName, JobWithTaskNames)>> {
	let jobs_dir = cfg_dir.join(JOBS_DIR_NAME);
	tracing::trace!("Searching for job configs in {jobs_dir:?}");

	WalkDir::new(&jobs_dir)
		.follow_links(true)
		.into_iter()
		.filter_map(move |dir_entry| {
			let job_config_path = dir_entry_is_job_config_file(&dir_entry)?;
			let job_name = JobName::from_job_config_path(job_config_path, &jobs_dir);

			// filter out all jobs that don't match the filter
			if let Some(filter) = filter {
				if !filter.iter().any(|filter| filter.job_matches(&job_name)) {
					tracing::trace!("Filtering out job {job_name:?}");
					return None;
				}
			}

			// parse the job from the config located at the config path
			let (job_name, mut job) = match get(job_config_path, job_name, cx).map_err(|e| {
				e.wrap_err(format!("invalid config at: {}", job_config_path.display()))
			}) {
				Ok(Some(job)) => job,
				Ok(None) => return None,
				Err(e) => return Some(Err(e)),
			};

			// when the job config doesn't contain any tasks, the global job settings are used to create a dummy task with no name
			assert!(
				!job.inner.tasks.is_empty(),
				"Jobs should always contain at least one task"
			);

			// filter out all tasks that don't match the filter
			if let Some(filter) = filter
				&& let Some(task_names) = &job.task_names
			{
				job.inner.tasks = job
					.inner
					.tasks
					.into_iter()
					.enumerate()
					.filter_map(|(idx, task)| {
						let task_name = task_names.get(&idx).expect(
							"task name map should always contain all task indecies and names",
						);

						if filter
							.iter()
							.any(|filter| filter.task_matches(&job_name, task_name))
						{
							Some(task)
						} else {
							tracing::trace!("Filtering out task {job_name:?}:{task_name:?}",);
							None
						}
					})
					.collect();

				if job.inner.tasks.is_empty() {
					let task_names_str = task_names.values().enumerate().fold(
						String::new(),
						|mut names_str, (idx, name)| {
							if idx == 0 {
								names_str.push_str(name);
							} else {
								_ = write!(names_str, ", {name}");
							}

							names_str
						},
					);

					tracing::warn!(
						"Asked to run job {job_name} but no tasks matched the task filter. Available tasks: {task_names_str}"
					);
					return None;
				}
			}

			Some(Ok((job_name, job)))
		})
}

#[tracing::instrument(skip(cx))]
pub fn get(path: &Path, name: JobName, cx: Context) -> Result<Option<(JobName, JobWithTaskNames)>> {
	tracing::trace!("Parsing a job from file");

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

/// Checks if the dir entry is a valid job config file
///
/// # Returns
/// The path of the job config file
fn dir_entry_is_job_config_file(dir_entry: &Result<DirEntry, walkdir::Error>) -> Option<&Path> {
	match dir_entry {
		Ok(dir_entry) => {
			// TODO: does this filter out directories?
			// filter out files with no extension
			let ext = dir_entry.path().extension()?;

			// or if the extension isn't CONFIG_FILE_EXT
			if ext != CONFIG_FILE_EXT {
				return None;
			}

			Some(dir_entry.path())
		}
		Err(err) => {
			match err.io_error().map(io::Error::kind) {
				Some(io::ErrorKind::NotFound) => (),
				_ => {
					tracing::warn!("File or directory is inaccessible: {err}");
				}
			}

			None
		}
	}
}
