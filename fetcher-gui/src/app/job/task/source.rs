/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod email;
pub mod exec;
pub mod file;
pub mod http;
pub mod reddit;
pub mod string;
pub mod twitter;

use crate::app::ScratchPad;
use fetcher_config::jobs::source::{
	email::Email, exec::Exec, file::File, http::Http, reddit::Reddit, string::StringSource,
	twitter::Twitter, Source,
};

use egui::{ComboBox, Ui};

pub fn show(ui: &mut Ui, source: &mut Option<Source>, scratch_pad: &mut ScratchPad) {
	ui.horizontal(|ui| {
		ui.label("Source:");
		ComboBox::from_id_source("source type")
			.wrap(false)
			.selected_text(source.as_ref().map_or("None".to_owned(), |x| x.to_string()))
			.show_ui(ui, |combo| {
				combo.selectable_value(source, None, "none");
				combo.selectable_value(
					source,
					Some(Source::String(StringSource::default())),
					"string",
				);
				combo.selectable_value(source, Some(Source::Http(Http(Vec::new()))), "http");
				combo.selectable_value(
					source,
					Some(Source::Twitter(Twitter::default())),
					"twitter",
				);
				combo.selectable_value(source, Some(Source::File(File(Vec::new()))), "file");
				combo.selectable_value(source, Some(Source::Reddit(Reddit::default())), "reddit");
				combo.selectable_value(source, Some(Source::Exec(Exec::default())), "exec");
				combo.selectable_value(source, Some(Source::Email(Email::default())), "email");
			})
	});

	if let Some(source) = source {
		ui.group(|ui| match source {
			Source::String(x) => string::show(ui, x),
			Source::Http(x) => http::show(ui, x, scratch_pad),
			Source::Twitter(x) => twitter::show(ui, x),
			Source::File(x) => file::show(ui, x, scratch_pad),
			Source::Reddit(x) => reddit::show(ui, x, scratch_pad),
			Source::Exec(x) => exec::show(ui, x),
			Source::Email(x) => email::show(ui, x, scratch_pad),
			Source::AlwaysErrors => todo!(),
		});
	}
}
