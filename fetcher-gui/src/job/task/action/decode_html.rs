/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::action::{decode_html::DecodeHtml, Field};

use egui::Ui;
use std::hash::Hash;

pub fn show(decode_html: &mut DecodeHtml, task_id: impl Hash, ui: &mut Ui) {
	for (idx, field) in decode_html.r#in.iter_mut().enumerate() {
		super::field::show(field, ("action decode html", idx, &task_id), ui);
	}

	ui.horizontal(|ui| {
		if ui.button("+").clicked() {
			decode_html.r#in.push(Field::default());
		}

		if ui.button("-").clicked() {
			decode_html.r#in.pop();
		}
	});
}
