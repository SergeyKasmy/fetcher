/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::action::Field;

use egui::{ComboBox, Ui};
use std::hash::Hash;

pub fn show(field: &mut Field, hash: impl Hash, ui: &mut Ui) {
	ComboBox::from_id_source(("field combobox", &hash))
		.selected_text(format!("{field:?}"))
		.show_ui(ui, |ui| {
			ui.selectable_value(field, Field::Title, "title");
			ui.selectable_value(field, Field::Body, "body");
			ui.selectable_value(field, Field::Link, "link");
			ui.selectable_value(field, Field::Id, "ID");
			ui.selectable_value(field, Field::ReplyTo, "reply_to");
			ui.selectable_value(field, Field::RawContents, "raw contents");
		});
}
