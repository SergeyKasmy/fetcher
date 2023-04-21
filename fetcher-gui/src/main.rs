/*
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod job;

use self::job::JobState;
use fetcher_config::jobs::{
	action::{
		contains::Contains, decode_html::DecodeHtml, extract::Extract, html::Html, import::Import,
		json::Json, remove_html::RemoveHtml, replace::Replace, set::Set, shorten::Shorten,
		take::Take, trim::Trim, use_as::Use, Action,
	},
	named::JobName,
	sink::Sink,
	task::Task,
	Job,
};

use eframe::NativeOptions;
use egui::{Color32, ScrollArea, SelectableLabel};
use std::collections::{BTreeMap, HashMap};

const COLOR_ERROR: Color32 = Color32::LIGHT_RED;

/// This macro makes the the enum contain the variant provided, either by matching it or by replacing it with a default one
#[macro_export]
macro_rules! get_state {
    (
		$current_state:expr, 
		$enum:ident::$desired_state:ident
	) => {{
		let current_state = $current_state;
		match current_state {
			$enum::$desired_state(inner) => inner,
			_ => {
				*current_state = $enum::$desired_state(Default::default());
				if let $enum::$desired_state(state) = current_state {
					state
				} else {
					unreachable!("Current state should've just been replaced with desired state");
				}
			}
		}
	}};
}

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
					tag: Some(format!("Tag of Task #0 of Job#{i}")),
					actions: Some(vec![
						Action::ReadFilter,
						Action::Take(Take::default()),
						Action::Contains(Contains::default()),
						Action::DebugPrint,
						Action::Feed,
						Action::Html(Html::default()),
						Action::Http,
						Action::Json(Json::default()),
						Action::Use(Use::default()),
						Action::Caps,
						Action::Set(Set::default()),
						Action::Shorten(Shorten::default()),
						Action::Trim(Trim::default()),
						Action::Replace(Replace::default()),
						Action::Extract(Extract::default()),
						Action::RemoveHtml(RemoveHtml::default()),
						Action::DecodeHtml(DecodeHtml::default()),
						Action::Sink(Sink::default()),
						Action::Import(Import::default()),
					]),
					..Default::default()
				},
			);

			tasks.insert(
				format!("Task #1 of Job#{i}").into(),
				Task {
					tag: Some(format!("Tag of Task #1 of Job#{i}")),
					..Default::default()
				},
			);

			(
				format!("Job #{i}").into(),
				Job {
					tasks: Some(tasks),
					..Default::default()
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
