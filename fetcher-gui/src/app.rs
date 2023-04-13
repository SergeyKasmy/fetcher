/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_config::jobs::{
	job::timepoint::TimePoint, named::JobName, read_filter, task::Task, Job,
};

use egui::SelectableLabel;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct App {
	pub jobs: BTreeMap<JobName, Job>,
	pub current_job: JobName,
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.top_level(ctx);
	}
}

impl App {
	fn top_level(&mut self, ctx: &egui::Context) {
		egui::SidePanel::left("left_panel").show(ctx, |ui| self.job_list_panel(ui));
		egui::CentralPanel::default().show(ctx, |ui| {
			property_panel(
				ui,
				(
					&self.current_job,
					self.jobs.get_mut(&self.current_job).unwrap(),
				),
			);
		});
	}

	fn job_list_panel(&mut self, ui: &mut egui::Ui) {
		egui::ScrollArea::vertical()
			.auto_shrink([false, false])
			.show(ui, |ui| {
				for job_name in self.jobs.keys() {
					// TODO: left align the text
					if ui
						.add_sized(
							[ui.available_width(), 0.0],
							SelectableLabel::new(&self.current_job == job_name, job_name.as_str()),
						)
						.clicked()
					{
						self.current_job = job_name.clone()
					}
				}
			});
	}
}

fn property_panel(ui: &mut egui::Ui, (name, job): (&JobName, &mut Job)) {
	ui.heading(name.as_str());

	let mut refresh = job
		.refresh
		.clone()
		.unwrap_or_else(|| TimePoint::At(String::new()));

	let mut refresh_val = match refresh.clone() {
		TimePoint::Every(x) => x,
		TimePoint::At(x) => x,
	};

	ui.horizontal(|ui| {
		ui.label("Refresh: ");

		if ui
			.radio(matches!(refresh, TimePoint::At(_)), "at")
			.clicked()
		{
			refresh = TimePoint::At(refresh_val.clone());
		}

		if ui
			.radio(matches!(refresh, TimePoint::Every(_)), "every")
			.clicked()
		{
			refresh = TimePoint::Every(refresh_val.clone());
		}

		ui.text_edit_singleline(&mut refresh_val);

		match &mut refresh {
			TimePoint::Every(x) => *x = refresh_val,
			TimePoint::At(x) => *x = refresh_val,
		}
	});
	job.refresh = Some(refresh);

	ui.heading("Tasks");

	for (task_name, task) in job.tasks.as_mut().unwrap().iter_mut() {
		ui.collapsing(task_name.as_str(), |ui| {
			task_properties(ui, task);
		});
	}
}
fn task_properties(ui: &mut egui::Ui, task: &mut Task) {
	ui.horizontal(|ui| {
		ui.label("Read Filter type: ");
		egui::ComboBox::from_id_source("read filter type")
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
		ui.label("Tag: ");
		ui.text_edit_singleline(&mut tag);
		task.tag = Some(tag);
	});

	ui.horizontal(|ui| {
		ui.label("Source:");
		ui.label(format!("{:?}", task.source));
	});

	ui.horizontal(|ui| {
		ui.label("Actions");
		ui.label(format!("{:?}", task.actions));
	});

	ui.horizontal(|ui| {
		let mut entry_to_msg_map_enabled = task.entry_to_msg_map_enabled.unwrap_or(false);
		ui.checkbox(
			&mut entry_to_msg_map_enabled,
			"Entry to message map enabled override: ",
		);
		task.entry_to_msg_map_enabled = Some(entry_to_msg_map_enabled);
	});

	ui.horizontal(|ui| {
		ui.label("Sink: ");
		ui.label(format!("{:?}", task.sink));
	});
}
