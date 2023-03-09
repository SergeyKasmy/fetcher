/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod timepoint;

use std::collections::HashMap;

use self::timepoint::TimePoint;
use super::{
	action::Action, external_data::ProvideExternalData, read_filter, sink::Sink, source::Source,
	task::Task, JobName, Tag, TaskName,
};
use crate::Error;
use fetcher_core::{job::Job as CJob, utils::OptionExt};

use serde::{Deserialize, Serialize};

pub type DisabledField = Option<bool>;
pub type TemplatesField = Option<Vec<String>>;

#[derive(Deserialize, Serialize, Debug)]
pub struct Job {
	#[serde(rename = "read_filter_type")]
	read_filter_kind: Option<read_filter::Kind>,
	tag: Option<Tag>,
	source: Option<Source>,
	#[serde(rename = "process")]
	actions: Option<Vec<Action>>,
	sink: Option<Sink>,

	tasks: Option<HashMap<TaskName, Task>>,
	refresh: Option<TimePoint>,

	// these are meant to be used externally and are unused here
	disabled: DisabledField,
	templates: TemplatesField,
}

impl Job {
	pub fn parse<D>(
		self,
		job: &JobName,
		external: &D,
	) -> Result<(CJob, Option<HashMap<usize, TaskName>>), Error>
	where
		D: ProvideExternalData + ?Sized,
	{
		match self.tasks {
			Some(mut tasks) if !tasks.is_empty() => {
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

					if task.sink.is_none() {
						task.sink = self.sink.clone();
					}
				}

				let tasks_and_task_name_map_iter =
					tasks.into_iter().enumerate().map(|(id, (name, task))| {
						let task = task.parse(job, Some(&name), external)?;

						Ok::<_, Error>((task, (id, name)))
					});

				// clippy false positive for iter.unzip()
				#[allow(clippy::redundant_closure_for_method_calls)]
				let (tasks, task_name_map): (Vec<_>, HashMap<usize, TaskName>) =
					itertools::process_results(tasks_and_task_name_map_iter, |iter| iter.unzip())?;

				let job = CJob {
					tasks,
					refresh_time: self.refresh.try_map(TimePoint::parse)?,
				};

				Ok((job, Some(task_name_map)))
			}
			// tasks is not set
			_ => {
				// copy paste all values from the job to a dummy task, i.e. create a single task with all the values from the job
				let task = Task {
					read_filter_kind: self.read_filter_kind,
					tag: self.tag,
					source: self.source,
					actions: self.actions,
					sink: self.sink,
				};

				Ok((
					CJob {
						tasks: vec![task.parse(job, None, external)?],
						refresh_time: self.refresh.try_map(TimePoint::parse)?,
					},
					None,
				))
			}
		}
	}
}
