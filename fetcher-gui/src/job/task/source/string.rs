/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::source::string::StringSource;

use egui::Ui;

pub fn show(ui: &mut Ui, StringSource(strings): &mut StringSource) {
	for s in &mut *strings {
		ui.text_edit_multiline(s);
	}

	ui.horizontal(|ui| {
		if ui.button("+").clicked() {
			strings.push(String::new());
		}

		if ui.button("-").clicked() && !strings.is_empty() {
			strings.remove(strings.len() - 1);
		}
	});
}
