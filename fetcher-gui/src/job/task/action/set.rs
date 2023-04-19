/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::field;
use fetcher_config::jobs::action::{set::Set, Field};

use egui::Ui;
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct SetState {
	pub new_field: Field,
}

impl SetState {
	pub fn show(&mut self, set: &mut Set, task_id: impl Hash, ui: &mut Ui) {
		for (idx, (field, values)) in set.0.iter_mut().enumerate() {
			if idx > 0 {
				ui.separator();
			}

			ui.heading(field.to_string());

			for value in values.iter_mut().flat_map(|x| x.0.iter_mut()) {
				ui.text_edit_singleline(value);
			}

			ui.horizontal(|ui| {
				if ui.button("+").clicked() {
					values
						.get_or_insert_with(Default::default)
						.0
						.push(String::new());
				}

				if ui.button("-").clicked() {
					if let Some(values_inner) = values {
						values_inner.0.pop();

						if values_inner.0.is_empty() {
							*values = None;
						}
					}
				}
			});
		}

		ui.horizontal(|ui| {
			if ui.button("+").clicked() {
				set.0.insert(self.new_field, None);
			}

			field::show(&mut self.new_field, ("action set edit field", &task_id), ui);

			if ui.button("-").clicked() {
				set.0.remove(&self.new_field);
			}
		});
	}
}
