/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use egui::Ui;

pub fn show(tag: &mut Option<String>, ui: &mut Ui) {
	ui.horizontal(|ui| {
		let tag_str = tag.get_or_insert_with(Default::default);

		ui.label("Tag:");
		ui.text_edit_singleline(tag_str);

		if tag_str.is_empty() {
			*tag = None;
		}
	});
}
