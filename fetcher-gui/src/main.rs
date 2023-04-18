/*
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod job;

use self::job::JobState;
use fetcher_config::jobs::{
	action::{take::Take, Action},
	named::JobName,
	task::Task,
	Job,
};

use eframe::NativeOptions;
use egui::{Color32, ScrollArea, SelectableLabel};
use std::collections::{BTreeMap, HashMap};

const COLOR_ERROR: Color32 = Color32::LIGHT_RED;

#[derive(Debug)]
pub struct App {
	pub current_job: JobName,
	pub jobs: BTreeMap<JobName, Job>,
	pub job_state: HashMap<JobName, JobState>,
}

fn main() {
	let jobs = (0..100)
		.map(|i| {
			let mut tasks = HashMap::new();

			tasks.insert(
				format!("Task #0 of Job#{i}").into(),
				Task {
					read_filter_kind: None,
					tag: Some(format!("Tag of Task #0 of Job#{i}")),
					source: None,
					actions: Some(vec![
						Action::Take(Take(fetcher_config::jobs::action::take::Inner {
							which: fetcher_config::jobs::action::take::TakeWhich::FromNewest,
							num: 1,
						})),
						Action::Contains(fetcher_config::jobs::action::contains::Contains(
							HashMap::new(),
						)),
						Action::DebugPrint,
						Action::Html(fetcher_config::jobs::action::html::Html {
							item: None,
							title: None,
							text: None,
							id: None,
							link: None,
							img: None,
						}),
					]),
					entry_to_msg_map_enabled: None,
					sink: None,
				},
			);
			tasks.insert(
				format!("Task #1 of Job#{i}").into(),
				Task {
					read_filter_kind: None,
					tag: Some(format!("Tag of Task #1 of Job#{i}")),
					source: None,
					actions: None,
					entry_to_msg_map_enabled: None,
					sink: None,
				},
			);

			(
				format!("Job #{i}").into(),
				Job {
					read_filter_kind: None,
					tag: None,
					source: None,
					actions: None,
					entry_to_msg_map_enabled: None,
					sink: None,
					tasks: Some(tasks),
					refresh: None,
					disabled: None,
					templates: None,
				},
			)
		})
		.collect::<BTreeMap<JobName, Job>>();

	eframe::run_native(
		"Configure fetcher",
		NativeOptions::default(),
		Box::new(|_ctx| {
			Box::new(App {
				current_job: jobs.first_key_value().unwrap().0.clone(),
				jobs,
				job_state: Default::default(),
			})
		}),
	)
	.unwrap();
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		self.top_level(ctx);
	}
}

impl App {
	fn top_level(&mut self, ctx: &egui::Context) {
		egui::SidePanel::left("job list side panel").show(ctx, |ui| {
			ScrollArea::vertical()
				.auto_shrink([false, false])
				.show(ui, |ui| {
					for job_name in self.jobs.keys() {
						// TODO: left align the text
						if ui
							.add_sized(
								[ui.available_width(), 0.0],
								SelectableLabel::new(
									&self.current_job == job_name,
									job_name.as_str(),
								),
							)
							.clicked()
						{
							self.current_job = job_name.clone()
						}
					}
				});
		});

		egui::CentralPanel::default().show(ctx, |ui| {
			ScrollArea::vertical().show(ui, |ui| {
				let job = self.jobs.entry(self.current_job.clone()).or_default();

				self.job_state
					.entry(self.current_job.clone())
					.or_default()
					.show(ui, self.current_job.clone(), job);
			});
		});
	}
}
