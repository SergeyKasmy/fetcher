/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{
	action::Action, external_data::ProvideExternalData, read_filter, sink::Sink, task::Task,
};
use crate::Error;
use fetcher_core::{job::Job as CoreJob, utils::OptionExt};

use serde::{Deserialize, Serialize};

pub type DisabledField = Option<bool>;
pub type TemplatesField = Option<Vec<String>>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Job {
	SingleTask(SingleTaskJob),
	SeveralTasks(SeveralTasksJob),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SingleTaskJob {
	refresh: Option<String>,
	#[serde(flatten)]
	task: Task,

	// these are meant to be used externally and are unused here
	disabled: DisabledField,
	templates: TemplatesField,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SeveralTasksJob {
	#[serde(rename = "read_filter_type")]
	read_filter_kind: Option<read_filter::Kind>,
	refresh: Option<String>,
	#[serde(rename = "process")]
	actions: Option<Vec<Action>>,
	sink: Option<Sink>,

	tasks: Vec<Task>,

	// these are meant to be used externally and are unused here
	disabled: DisabledField,
	templates: TemplatesField,
}

impl Job {
	pub fn parse(self, name: &str, external: &dyn ProvideExternalData) -> Result<CoreJob, Error> {
		match self {
			Job::SingleTask(x) => x.parse(name, external),
			Job::SeveralTasks(x) => x.parse(name, external),
		}
	}
}

impl SingleTaskJob {
	pub fn parse(self, name: &str, external: &dyn ProvideExternalData) -> Result<CoreJob, Error> {
		Ok(CoreJob {
			tasks: vec![self.task.parse(name, external)?],
			refetch_interval: self.refresh.try_map(duration_str::parse_std)?,
		})
	}
}

impl SeveralTasksJob {
	pub fn parse(
		mut self,
		name: &str,
		external: &dyn ProvideExternalData,
	) -> Result<CoreJob, Error> {
		for task in &mut self.tasks {
			task.read_filter_kind = task.read_filter_kind.or(self.read_filter_kind);

			if task.actions.is_none() {
				task.actions = self.actions.clone();
			}

			if task.sink.is_none() {
				task.sink = self.sink.clone();
			}
		}

		Ok(CoreJob {
			tasks: self
				.tasks
				.into_iter()
				.map(|x| x.parse(name, external))
				.collect::<Result<Vec<_>, _>>()?,
			refetch_interval: self.refresh.try_map(duration_str::parse_std)?,
		})
	}
}
