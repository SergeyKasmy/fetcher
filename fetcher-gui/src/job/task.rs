/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod action;
pub mod read_filter_type;
pub mod source;
pub mod tag;

use self::{
	action::{sink::SinkState, ActionEditorState},
	source::SourceState,
};
use fetcher_config::jobs::task::Task;

use egui::{Ui, Window};
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct TaskState {
	pub source_state: SourceState,
	pub is_actions_editor_shown: bool,
	pub actions_state: Option<ActionEditorState>,
	pub sink_state: SinkState,
}

impl TaskState {
	pub fn show(&mut self, task: &mut Task, task_id: impl Hash, ui: &mut Ui) {
		read_filter_type::show(&mut task.read_filter_kind, &task_id, ui);
		tag::show(&mut task.tag, ui);
		self.source_state.show(&mut task.source, &task_id, ui);

		if ui
			.button(format!(
				"Actions: {}",
				task.actions.as_ref().map_or(0, Vec::len)
			))
			.clicked()
		{
			self.is_actions_editor_shown = true;
		}

		Window::new("Actions edit")
			.id(egui::Id::new(("actions editor", &task_id)))
			.open(&mut self.is_actions_editor_shown)
			.show(ui.ctx(), |ui| {
				self.actions_state
					.get_or_insert_with(|| ActionEditorState::new(task.actions.as_deref()))
					.show(&mut task.actions, &task_id, ui);
			});

		ui.checkbox(
			&mut task.entry_to_msg_map_enabled,
			"Entry to message map enabled override",
		);

		ui.label("Sink:");
		if let Some(sink) = &mut task.sink {
			self.sink_state.show(sink, &task_id, ui);
		}
	}
}
