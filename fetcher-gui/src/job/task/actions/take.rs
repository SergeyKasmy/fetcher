/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::COLOR_ERROR;
use fetcher_config::jobs::action::take::{Take, TakeWhich};

use egui::{ComboBox, Ui};
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct TakeState {
	pub num: Option<String>,
}

impl TakeState {
	pub fn show(&mut self, take: &mut Take, task_id: impl Hash, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.label("Take from");
			ComboBox::from_id_source(("action take from combo box", task_id))
				.selected_text(format!("{:?}", take.0.which))
				.show_ui(ui, |ui| {
					ui.selectable_value(&mut take.0.which, TakeWhich::FromNewest, "Newest");
					ui.selectable_value(&mut take.0.which, TakeWhich::FromOldest, "Oldest");
				});
		});

		ui.horizontal(|ui| {
			ui.label("Number:");

			let num_str = self.num.get_or_insert_with(|| take.0.num.to_string());
			ui.text_edit_singleline(num_str);

			match num_str.parse::<usize>() {
				Ok(num) => take.0.num = num,
				Err(_) => {
					ui.colored_label(COLOR_ERROR, "Not a valid number");
				}
			}
		});
	}
}
