/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::read_filter;

use egui::{ComboBox, Ui};
use std::hash::Hash;

pub fn show(read_filter_kind: &mut Option<read_filter::Kind>, task_id: impl Hash, ui: &mut Ui) {
	ui.horizontal(|ui| {
		ui.label("Read Filter type:");
		ComboBox::from_id_source(("read filter type", &task_id))
			.wrap(false)
			.selected_text(format!("{read_filter_kind:?}"))
			.show_ui(ui, |combo| {
				combo.selectable_value(read_filter_kind, None, "none");
				combo.selectable_value(
					read_filter_kind,
					Some(read_filter::Kind::NewerThanRead),
					"newer than read",
				);
				combo.selectable_value(
					read_filter_kind,
					Some(read_filter::Kind::NotPresentInReadList),
					"not present in read list",
				);
			})
	});
}
