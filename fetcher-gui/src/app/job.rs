/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod task;

use self::task::TaskState;
use fetcher_config::jobs::{
	job::timepoint::TimePoint,
	named::{JobName, TaskName},
	Job,
};

use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct JobState {
	pub task_state: HashMap<TaskName, TaskState>,
}

impl JobState {
	pub fn show(&mut self, ui: &mut egui::Ui, name: JobName, job: &mut Job) {
		ui.heading(name.as_str());

		let mut refresh = job
			.refresh
			.clone()
			.unwrap_or_else(|| TimePoint::At(String::new()));

		let mut refresh_val = match refresh.clone() {
			TimePoint::Every(x) => x,
			TimePoint::At(x) => x,
		};

		ui.horizontal(|ui| {
			ui.label("Refresh:");

			if ui
				.radio(matches!(refresh, TimePoint::At(_)), "at")
				.clicked()
			{
				refresh = TimePoint::At(refresh_val.clone());
			}

			if ui
				.radio(matches!(refresh, TimePoint::Every(_)), "every")
				.clicked()
			{
				refresh = TimePoint::Every(refresh_val.clone());
			}

			ui.text_edit_singleline(&mut refresh_val);

			match &mut refresh {
				TimePoint::Every(x) => *x = refresh_val,
				TimePoint::At(x) => *x = refresh_val,
			}
		});
		job.refresh = Some(refresh);

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
