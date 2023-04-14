/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::app::ScratchPad;
use fetcher_config::jobs::source::file::File;

use egui::Ui;
use std::path::PathBuf;

pub fn show(ui: &mut Ui, File(paths): &mut File, scratch_pad: &mut ScratchPad) {
	for (idx, path) in paths.iter_mut().enumerate() {
		let path_str = scratch_pad
			.entry(format!("source.file.{idx}"))
			.or_insert_with(|| path.to_string_lossy().into_owned());

		ui.text_edit_singleline(path_str);

		*path = PathBuf::from(path_str.clone());
	}

	ui.horizontal(|ui| {
		if ui.button("+").clicked() {
			paths.push(PathBuf::new());
		}

		if ui.button("-").clicked() && !paths.is_empty() {
			paths.remove(paths.len() - 1);
		}
	});
}
