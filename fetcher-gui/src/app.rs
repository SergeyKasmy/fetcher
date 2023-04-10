/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use egui::SelectableLabel;

#[derive(Default, Debug)]
pub struct App {
	pub job_list: Vec<String>,
	pub current_job: String,
	pub refresh: String,
	pub read_filter_type: ReadFilterType,
	pub source: String,
	pub actions: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub enum ReadFilterType {
	#[default]
	NewerThanRead,
	NotPresentInReadList,
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.top_level(ctx);
	}
}
impl App {
	fn top_level(&mut self, ctx: &egui::Context) {
		egui::SidePanel::left("left_panel").show(ctx, |ui| self.job_list_panel(ui));
		egui::CentralPanel::default().show(ctx, |ui| self.property_panel(ui));
	}

	fn job_list_panel(&mut self, ui: &mut egui::Ui) {
		egui::ScrollArea::vertical()
			.auto_shrink([false, false])
			.show(ui, |ui| {
				for job in &self.job_list {
					// TODO: left align the text
					if ui
						.add_sized(
							[ui.available_width(), 0.0],
							SelectableLabel::new(&self.current_job == job, job),
						)
						.clicked()
					{
						self.current_job = job.clone()
					}
				}
			});
	}

	fn property_panel(&mut self, ui: &mut egui::Ui) {
		ui.heading(&self.current_job);

		ui.horizontal(|ui| {
			ui.label("Refresh: ");
			ui.text_edit_singleline(&mut self.refresh);
		});

		ui.horizontal(|ui| {
			ui.label("Read Filter type: ");
			egui::ComboBox::from_id_source("read filter type")
				.wrap(false)
				.selected_text(format!("{:?}", self.read_filter_type))
				.show_ui(ui, |combo| {
					combo.selectable_value(
						&mut self.read_filter_type,
						ReadFilterType::NewerThanRead,
						"newer than read",
					);
					combo.selectable_value(
						&mut self.read_filter_type,
						ReadFilterType::NotPresentInReadList,
						"not present in read list",
					);
				})
		});

		ui.horizontal(|ui| {
			ui.label("Source");
			ui.text_edit_singleline(&mut self.source);
		});

		ui.horizontal(|ui| {
			ui.label("Actions");
			ui.text_edit_singleline(&mut self.actions);
		});
	}
}
