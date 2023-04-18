/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::action::Action;

use egui::{panel::Side, CentralPanel, ScrollArea, SelectableLabel, SidePanel, TopBottomPanel, Ui};
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct ActionsState {
	pub current_action_idx: Option<usize>,
}

impl ActionsState {
	pub fn show(&mut self, actions: &mut Option<Vec<Action>>, task_id: impl Hash, ui: &mut Ui) {
		SidePanel::new(Side::Left, egui::Id::new(("actions list", &task_id))).show_inside(
			ui,
			|ui| {
				ScrollArea::vertical().show(ui, |ui| {
					for (idx, act) in actions.iter().flatten().enumerate() {
						// TODO: left align the text
						if ui
							.add_sized(
								[ui.available_width(), 0.0],
								SelectableLabel::new(
									*self.current_action_idx.get_or_insert(0) == idx,
									act.to_string(),
								),
							)
							.clicked()
						{
							self.current_action_idx = Some(idx);
						}
					}
				});
			},
		);

		// NOTE: fixes a bug in egui that makes the CentralPanel below overflow the window.
		// See https://github.com/emilk/egui/issues/901
		TopBottomPanel::bottom(egui::Id::new((
			"actions list invisible bottom panel",
			task_id,
		)))
		.show_separator_line(false)
		.show_inside(ui, |_| ());

		CentralPanel::default().show_inside(ui, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				if let Some((idx, _action)) = self.current_action_idx.zip(actions.as_mut()) {
					ui.heading(format!("CURRENT ACTION: #{idx}"));
				}
			});
		});
	}
}
