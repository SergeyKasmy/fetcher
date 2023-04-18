/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::action::{contains::Contains, Field};

use egui::{ComboBox, Ui};
use std::hash::Hash;

#[derive(Debug)]
pub struct ContainsState {
	pub selected_field: Field,
}

impl ContainsState {
	pub fn show(&mut self, contains: &mut Contains, task_id: impl Hash, ui: &mut Ui) {
		for (field, regex) in &mut contains.0 {
			ui.horizontal(|ui| {
				ui.label(field.to_string());
				ui.text_edit_singleline(regex);
			});
		}

		ui.horizontal(|ui| {
			if ui.button("+").clicked() {
				contains.0.entry(self.selected_field.clone()).or_default();
			}

			ComboBox::from_id_source(("action contains field", task_id))
				.selected_text(self.selected_field.to_string())
				.show_ui(ui, |ui| {
					ui.selectable_value(&mut self.selected_field, Field::Title, "title");
					ui.selectable_value(&mut self.selected_field, Field::Body, "body");
					ui.selectable_value(&mut self.selected_field, Field::Link, "link");
					ui.selectable_value(&mut self.selected_field, Field::Id, "ID");
					ui.selectable_value(&mut self.selected_field, Field::ReplyTo, "reply_to");
					ui.selectable_value(
						&mut self.selected_field,
						Field::RawContents,
						"raw contents",
					);
				});

			if ui.button("-").clicked() {
				contains.0.remove(&self.selected_field);
			}
		});
	}
}

impl Default for ContainsState {
	fn default() -> Self {
		Self {
			selected_field: Field::Title,
		}
	}
}
