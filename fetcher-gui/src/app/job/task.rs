/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod source;

use crate::app::ScratchPad;
use fetcher_config::jobs::{read_filter, task::Task};

use egui::{ComboBox, Ui};

pub fn show(ui: &mut Ui, task: &mut Task, scratch_pad: &mut ScratchPad) {
	ui.horizontal(|ui| {
		ui.label("Read Filter type:");
		ComboBox::from_id_source("read filter type")
			.wrap(false)
			.selected_text(format!("{:?}", task.read_filter_kind))
			.show_ui(ui, |combo| {
				combo.selectable_value(&mut task.read_filter_kind, None, "none");
				combo.selectable_value(
					&mut task.read_filter_kind,
					Some(read_filter::Kind::NewerThanRead),
					"newer than read",
				);
				combo.selectable_value(
					&mut task.read_filter_kind,
					Some(read_filter::Kind::NotPresentInReadList),
					"not present in read list",
				);
			})
	});

	ui.horizontal(|ui| {
		let mut tag = task.tag.clone().unwrap();
		ui.label("Tag:");
		ui.text_edit_singleline(&mut tag);
		task.tag = Some(tag);
	});

	source::show(ui, &mut task.source, scratch_pad);

	ui.horizontal(|ui| {
		ui.label("Actions");
		ui.label(format!("{:?}", task.actions));
	});

	ui.horizontal(|ui| {
		let mut entry_to_msg_map_enabled = task.entry_to_msg_map_enabled.unwrap_or(false);
		ui.checkbox(
			&mut entry_to_msg_map_enabled,
			"Entry to message map enabled override",
		);
		task.entry_to_msg_map_enabled = Some(entry_to_msg_map_enabled);
	});

	ui.horizontal(|ui| {
		ui.label("Sink:");
		ui.label(format!("{:?}", task.sink));
	});
}
