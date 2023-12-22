/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod timepoint;

use self::timepoint::TimePoint;
use super::{
	action::Action,
	external_data::ProvideExternalData,
	named::{JobName, JobWithTaskNames, TaskName},
	read_filter,
	sink::Sink,
	source::Source,
	task::Task,
};
use crate::FetcherConfigError;
use fetcher_core::{job::Job as CJob, utils::OptionExt};

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ops::Not};

pub type DisabledField = Option<bool>;
pub type TemplatesField = Option<Vec<String>>;

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Job {
	#[serde(rename = "read_filter_type")]
	pub read_filter_kind: Option<read_filter::Kind>,
	pub tag: Option<String>,
	pub source: Option<Source>,
	#[serde(rename = "process")]
	pub actions: Option<Vec<Action>>,

	#[serde(default)]
	pub entry_to_msg_map_enabled: bool,
	pub sink: Option<Sink>,

	pub tasks: Option<HashMap<TaskName, Task>>,
	pub refresh: Option<TimePoint>,

	// these are meant to be used externally and are unused here
	pub disabled: DisabledField,
	pub templates: TemplatesField,
}

impl Job {
	// TODO: remove unnecessary JobName return value (it's already passed as an argument). There's no point in returning it back. If they wanted to keep it, they could've just cloned it
	pub fn parse<D>(
		mut self,
		name: JobName,
		external: &D,
	) -> Result<(JobName, JobWithTaskNames), FetcherConfigError>
	where
		D: ProvideExternalData + ?Sized,
	{
		match self.tasks.take() {
			Some(tasks) if !tasks.is_empty() => self.parse_with_tasks_map(name, tasks, external),
			// tasks is not set
			_ => {
				// copy paste all values from the job to a dummy task, i.e. create a single task with all the values from the job
				let task = Task {
					read_filter_kind: self.read_filter_kind,
					tag: self.tag,
					source: self.source,
					actions: self.actions,
					entry_to_msg_map_enabled: self.entry_to_msg_map_enabled,
					sink: self.sink,
				};

				let job = CJob {
					tasks: vec![task.parse(&name, None, external)?],
					refresh_time: self.refresh.try_map(TimePoint::parse)?,
				};

				Ok((
					name,
					JobWithTaskNames {
						inner: job,
						task_names: None,
					},
				))
			}
		}
	}

	/// ignores self.tasks and uses tasks parameter instead
	fn parse_with_tasks_map<D>(
		self,
		name: JobName,
		mut tasks: HashMap<TaskName, Task>,
		external: &D,
	) -> Result<(JobName, JobWithTaskNames), FetcherConfigError>
	where
		D: ProvideExternalData + ?Sized,
	{
		tracing::trace!("Parsing job {name:?} with tasks {tasks:#?}");

		// append values from the job if they are not present in the tasks
		for task in tasks.values_mut() {
			task.read_filter_kind = task.read_filter_kind.or(self.read_filter_kind);

			if task.tag.is_none() {
				task.tag = self.tag.clone();
			}

			if task.source.is_none() {
				task.source = self.source.clone();
			}

			if task.actions.is_none() {
				task.actions = self.actions.clone();
			}

			if self.entry_to_msg_map_enabled {
				task.entry_to_msg_map_enabled = true;
			}

			if task.sink.is_none() {
				task.sink = self.sink.clone();
			}
		}

		// FIXME: broken. Filtering can remove tasks from the tasks map. Then, when checking if we should pass the task name as a tag, we ignore the fact that we could've had more tasks in the job and skip the tag which we shouldn't do
		// TODO: add disabled field to the task and enable that instead of removing tasks when filtering outright. Then the code below will work again
		/*
		// NOTE: if the tasks map only has one task, don't provide the task its name to avoid it automatically adding a tag
		// a tag should only be automatically added if there are more then 1 task in the tasks map
		let single_task = tasks.len() == 1;

		if single_task {
			tracing::trace!("Not setting task name as tag in a tasks map with a single element");
		}
		*/
		let single_task = false; // remove when above is fixed

		let tasks_and_task_name_map_iter =
			tasks
				.into_iter()
				.enumerate()
				.map(|(id, (task_name, task))| {
					let task =
						task.parse(&name, single_task.not().then_some(&task_name), external)?;

					Ok::<_, FetcherConfigError>((task, (id, task_name)))
				});

		// clippy false positive for iter.unzip()
		#[allow(clippy::redundant_closure_for_method_calls)]
		let (tasks, task_names): (Vec<_>, HashMap<usize, TaskName>) =
			itertools::process_results(tasks_and_task_name_map_iter, |iter| iter.unzip())?;

		let job = CJob {
			tasks,
			refresh_time: self.refresh.try_map(TimePoint::parse)?,
		};

		Ok((
			name,
			JobWithTaskNames {
				inner: job,
				task_names: Some(task_names),
			},
		))
	}
}
