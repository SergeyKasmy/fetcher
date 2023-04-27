/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod refresh;
pub mod task;

use self::{
	refresh::RefreshState,
	task::{
		action::{sink::SinkState, ActionEditorState},
		source::SourceState,
		TaskState,
	},
};
use egui::Window;
use fetcher_config::jobs::{
	named::{JobName, TaskName},
	Job,
};

use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct JobState {
	pub refresh_state: RefreshState,
	pub task_state: HashMap<TaskName, TaskState>,
	pub source_state: SourceState,
	pub is_actions_editor_shown: bool,
	pub actions_state: Option<ActionEditorState>,
	pub sink_state: SinkState,
}

impl JobState {
	pub fn show(&mut self, name: &JobName, job: &mut Job, ui: &mut egui::Ui) {
		ui.heading(name.as_str());

		self.refresh_state.show(&mut job.refresh, ui);

		ui.heading("Tasks");

		for (task_name, task) in job
			.tasks
			.as_mut()
			.map(HashMap::iter_mut)
			.into_iter()
			.flatten()
		{
			ui.collapsing(task_name.as_str(), |ui| {
				self.task_state
					.entry(task_name.clone())
					.or_default()
					.show(task, task_name, ui);
			});
		}

		ui.separator();

		// FIXME: don't only show shared settings when they exist
		ui.heading("Shared settings");

		if job.tag.is_some() {
			task::tag::show(&mut job.tag, ui);
		}

		if job.read_filter_kind.is_some() {
			task::read_filter_type::show(&mut job.read_filter_kind, name, ui);
		}

		if job.source.is_some() {
			self.source_state.show(&mut job.source, name, ui);
		}

		if job.actions.is_some() {
			if ui
				.button(format!(
					"Actions: {}",
					job.actions.as_ref().map_or(0, Vec::len)
				))
				.clicked()
			{
				self.is_actions_editor_shown = true;
			}

			Window::new("Actions edit")
				.id(egui::Id::new(("actions editor", &name)))
				.open(&mut self.is_actions_editor_shown)
				.show(ui.ctx(), |ui| {
					self.actions_state
						.get_or_insert_with(|| ActionEditorState::new(job.actions.as_deref()))
						.show(&mut job.actions, name, ui);
				});
		}

		if job.entry_to_msg_map_enabled {
			ui.checkbox(
				&mut job.entry_to_msg_map_enabled,
				"Entry to message map enabled override",
			);
		}

		if let Some(sink) = &mut job.sink {
			ui.label("Sink:");
			self.sink_state.show(sink, name, ui);
		}
	}
}
