/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod action;
pub mod source;

use self::{action::ActionEditorState, source::SourceState};
use fetcher_config::jobs::{read_filter, task::Task};

use egui::{ComboBox, Ui, Window};
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct TaskState {
	pub source_state: SourceState,
	pub is_actions_editor_shown: bool,
	pub actions_state: Option<ActionEditorState>,
}

impl TaskState {
	pub fn show(&mut self, task: &mut Task, task_id: impl Hash, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.label("Read Filter type:");
			ComboBox::from_id_source(("read filter type", &task_id))
				.wrap(false)
				.selected_text(format!("{:?}", task.read_filter_kind))
				.show_ui(ui, |combo| {
					combo.selectable_value(&mut task.read_filter_kind, None, "none");
					combo.selectable_value(
						&mut task.read_filter_kind,
						Some(read_filter::Kind::NewerThanRead),
						"newer than read",
					);
					combo.selectable_value(
						&mut task.read_filter_kind,
						Some(read_filter::Kind::NotPresentInReadList),
						"not present in read list",
					);
				})
		});

		ui.horizontal(|ui| {
			let tag = task.tag.get_or_insert_with(Default::default);

			ui.label("Tag:");
			ui.text_edit_singleline(tag);

			if tag.is_empty() {
				task.tag = None;
			}
		});

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

		// ui.horizontal(|ui| {
		// 	ui.label("Sink:");
		// 	ui.label(format!("{:?}", task.sink));
		// });
	}
}
