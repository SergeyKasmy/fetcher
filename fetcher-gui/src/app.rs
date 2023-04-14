/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod job;

use fetcher_config::jobs::{named::JobName, Job};

use egui::{Color32, ScrollArea, SelectableLabel};
use std::collections::{BTreeMap, HashMap};

const COLOR_ERROR: Color32 = Color32::LIGHT_RED;

pub type ScratchPad<Id = String> = HashMap<Id, String>;

#[derive(Debug)]
pub struct App {
	pub jobs: BTreeMap<JobName, Job>,
	pub current_job: JobName,
	pub scratch_pad: ScratchPad,
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.top_level(ctx);
	}
}

impl App {
	fn top_level(&mut self, ctx: &egui::Context) {
		egui::SidePanel::left("job list side panel").show(ctx, |ui| self.job_list_panel(ui));
		egui::CentralPanel::default().show(ctx, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				job::show(
					ui,
					self.current_job.clone(),
					self.jobs.get_mut(&self.current_job).unwrap(),
					&mut self.scratch_pad,
				);
			});
		});
	}

	fn job_list_panel(&mut self, ui: &mut egui::Ui) {
		ScrollArea::vertical()
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
