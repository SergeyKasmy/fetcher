/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::source::exec::Exec;

use egui::Ui;

pub fn show(ui: &mut Ui, Exec { cmd: cmds }: &mut Exec) {
	for cmd in &mut *cmds {
		ui.text_edit_multiline(cmd);
	}

	ui.horizontal(|ui| {
		if ui.button("+").clicked() {
			cmds.push(String::new());
		}

		if ui.button("-").clicked() && !cmds.is_empty() {
			cmds.remove(cmds.len() - 1);
		}
	});
}
