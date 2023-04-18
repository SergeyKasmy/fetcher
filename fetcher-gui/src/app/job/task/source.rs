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

use self::{file::FileState, http::HttpState, reddit::RedditState};
use fetcher_config::jobs::source::{
	email::Email, exec::Exec, file::File, http::Http, reddit::Reddit, string::StringSource,
	twitter::Twitter, Source,
};

use egui::{ComboBox, Ui, Window};
use std::hash::Hash;

#[derive(Default, Debug)]
pub struct SourceState {
	pub http_state: HttpState,
	pub file_state: FileState,
	pub reddit_state: RedditState,
	pub is_edit_window_shown: bool,
}

impl SourceState {
	pub fn show(&mut self, source: &mut Option<Source>, task_id: impl Hash, ui: &mut Ui) {
		ui.horizontal(|ui| {
			ui.label("Source:");
			ComboBox::from_id_source(("source type", &task_id))
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
					combo.selectable_value(
						source,
						Some(Source::Reddit(Reddit::default())),
						"reddit",
					);
					combo.selectable_value(source, Some(Source::Exec(Exec::default())), "exec");
					combo.selectable_value(source, Some(Source::Email(Email::default())), "email");
				});

			if let Some(source) = source {
				if ui.button("edit").clicked() {
					self.is_edit_window_shown = true;
				}

				Window::new("Source edit")
					.id(egui::Id::new(("source editor", &task_id)))
					// .anchor(Align2::CENTER_CENTER, Vec2::default())
					// .collapsible(false)
					// .movable(false)
					// .resizable(false)
					.open(&mut self.is_edit_window_shown)
					.show(ui.ctx(), |ui| match source {
						Source::String(x) => string::show(ui, x),
						Source::Http(x) => self.http_state.show(x, ui),
						Source::Twitter(x) => twitter::show(ui, x),
						Source::File(x) => self.file_state.show(x, ui),
						Source::Reddit(x) => self.reddit_state.show(x, task_id, ui),
						Source::Exec(x) => exec::show(ui, x),
						Source::Email(x) => email::show(x, task_id, ui),
						Source::AlwaysErrors => todo!(),
					});
			}
		});
	}
}
