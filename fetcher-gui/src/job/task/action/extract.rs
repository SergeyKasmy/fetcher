/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::field;
use fetcher_config::jobs::action::extract::Extract;

use egui::Ui;
use std::hash::Hash;

pub fn show(extract: &mut Extract, task_id: impl Hash, ui: &mut Ui) {
	ui.horizontal(|ui| {
		ui.label("From field");
		field::show(
			&mut extract.from_field,
			("action extract from field", task_id),
			ui,
		);
	});

	ui.horizontal(|ui| {
		ui.label("Regex");
		ui.text_edit_singleline(&mut extract.re);
	});

	ui.checkbox(
		&mut extract.passthrough_if_not_found,
		"Passthrough if not found",
	);
}
