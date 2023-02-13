/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{
	action::Action, external_data::ProvideExternalData, read_filter, sink::Sink, task::Task,
};
use crate::Error;
use fetcher_core::job::Job as CoreJob;

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub type DisabledField = Option<bool>;
pub type TemplatesField = Option<Vec<String>>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Job {
	#[serde(rename = "read_filter_type")]
	read_filter_kind: Option<read_filter::Kind>,
	// tag: Option<String>,
	refresh: Option<u64>,
	#[serde(rename = "process")]
	actions: Option<Vec<Action>>,
	// TODO: several sinks or integrate into actions
	sink: Option<Sink>,

	tasks: Vec<Task>,

	// these are meant to be used externally and are unused here
	disabled: DisabledField,
	templates: TemplatesField,
}

impl Job {
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
			refetch_interval: self
				.refresh
				.map(|i| Duration::from_secs(i * 60 /* secs in a min */)),
		})
	}
}
