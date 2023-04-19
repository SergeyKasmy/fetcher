/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::action::{
	use_as::{As, Use},
	Field,
};

use egui::Ui;
use std::hash::Hash;

use super::field;

#[derive(Default, Debug)]
pub struct UseState {
	pub new_field: Field,
}

impl UseState {
	pub fn show(&mut self, use_as: &mut Use, task_id: impl Hash, ui: &mut Ui) {
		for (idx, (field, As { r#as: az })) in use_as.0.iter_mut().enumerate() {
			ui.horizontal(|ui| {
				ui.label(field.to_string() + " as");

				field::show(az, ("action use as", idx, &task_id), ui);
			});
		}

		ui.horizontal(|ui| {
			if ui.button("+").clicked() {
				use_as.0.insert(
					self.new_field,
					As {
						r#as: Field::default(),
					},
				);
			}

			field::show(&mut self.new_field, ("action use edit field", &task_id), ui);

			if ui.button("-").clicked() {
				use_as.0.remove(&self.new_field);
			}
		});
	}
}
