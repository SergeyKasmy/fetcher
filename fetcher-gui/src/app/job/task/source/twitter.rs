/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::source::twitter::Twitter;

use egui::Ui;

pub fn show(ui: &mut Ui, Twitter(handles): &mut Twitter) {
	for handle in &mut *handles {
		ui.horizontal(|ui| {
			ui.label("@");
			ui.text_edit_singleline(handle);
		});
	}

	ui.horizontal(|ui| {
		if ui.button("+").clicked() {
			handles.push(String::new());
		}

		if ui.button("-").clicked() && !handles.is_empty() {
			handles.remove(handles.len() - 1);
		}
	});
}
