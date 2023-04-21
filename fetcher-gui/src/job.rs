/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod refresh;
pub mod task;

use self::{refresh::RefreshState, task::TaskState};
use fetcher_config::jobs::{
	named::{JobName, TaskName},
	Job,
};

use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct JobState {
	pub refresh_state: RefreshState,
	pub task_state: HashMap<TaskName, TaskState>,
}

impl JobState {
	pub fn show(&mut self, ui: &mut egui::Ui, name: JobName, job: &mut Job) {
		ui.heading(name.as_str());

		self.refresh_state.show(&mut job.refresh, ui);

		ui.heading("Tasks");

		for (idx, (task_name, task)) in job.tasks.as_mut().unwrap().iter_mut().enumerate() {
			if idx > 0 {
				ui.separator();
			}

			self.task_state
				.entry(task_name.clone())
				.or_default()
				.show(task, task_name, ui);
		}
	}
}

