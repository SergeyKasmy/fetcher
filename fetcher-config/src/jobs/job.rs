/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod timepoint;

use self::timepoint::TimePoint;
use super::{
	action::Action, external_data::ProvideExternalData, read_filter, sink::Sink, source::Source,
	task::Task,
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
	name: Option<String>,
	source: Option<Source>,
	#[serde(rename = "process")]
	actions: Option<Vec<Action>>,
	sink: Option<Sink>,

	tasks: Option<Vec<Task>>,
	refresh: Option<TimePoint>,

	// these are meant to be used externally and are unused here
	disabled: DisabledField,
	templates: TemplatesField,
}

impl Job {
	pub fn parse<D>(self, name: &str, external: &D) -> Result<CJob, Error>
	where
		D: ProvideExternalData + ?Sized,
	{
		match self.tasks {
			Some(mut tasks) if !tasks.is_empty() => {
				// append values from the job if they are not present in the tasks
				for task in &mut tasks {
					task.read_filter_kind = task.read_filter_kind.or(self.read_filter_kind);

					if task.name.is_none() {
						task.name = self.name.clone();
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

				Ok(CJob {
					tasks: tasks
						.into_iter()
						.map(|x| x.parse(name, external))
						.collect::<Result<Vec<_>, _>>()?,
					refresh_time: self.refresh.try_map(TimePoint::parse)?,
				})
			}
			// tasks is not set
			_ => {
				// copy paste all values from the job to a dummy task, i.e. create a single task with all the values from the job
				let task = Task {
					read_filter_kind: self.read_filter_kind,
					name: self.name,
					source: self.source,
					actions: self.actions,
					sink: self.sink,
				};

				Ok(CJob {
					tasks: vec![task.parse(name, external)?],
					refresh_time: self.refresh.try_map(TimePoint::parse)?,
				})
			}
		}
	}
}
