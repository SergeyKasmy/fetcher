/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::field;
use fetcher_config::jobs::action::replace::Replace;

use egui::Ui;
use std::hash::Hash;

pub fn show(replace: &mut Replace, task_id: impl Hash, ui: &mut Ui) {
	ui.heading("Replace");

	ui.horizontal(|ui| {
		ui.label("Regex");
		ui.text_edit_singleline(&mut replace.re);
	});

	ui.horizontal(|ui| {
		ui.label("in field");
		field::show(
			&mut replace.in_field,
			("action replace in field", task_id),
			ui,
		);
	});

	ui.horizontal(|ui| {
		ui.label("With");
		ui.text_edit_singleline(&mut replace.with);
	});
}
