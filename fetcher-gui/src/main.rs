/*
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// Hand selected lints
//#![warn(missing_docs)]  // TODO: add more docs
// TODO: add #![deny(clippy::unwrap_used)]
#![forbid(unsafe_code)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::exit)]
#![warn(clippy::filetype_is_file)]
#![warn(clippy::format_push_string)]
#![warn(clippy::let_underscore_untyped)]
#![warn(clippy::missing_assert_message)]
// #![warn(clippy::missing_docs_in_private_items)]	// TODO: enable later
#![warn(clippy::print_stderr)]
#![warn(clippy::rest_pat_in_fully_bound_structs)]
#![warn(clippy::same_name_method)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_to_string)]
#![warn(clippy::tests_outside_test_module)]
#![warn(clippy::todo)]
#![warn(clippy::try_err)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unimplemented)]
// Additional automatic Lints
#![warn(clippy::pedantic)]
// some types are more descriptive with modules name in the name, especially if this type is often used out of the context of this module
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::nursery)]
#![allow(clippy::option_if_let_else)] // "harder to read, false branch before true branch"
#![allow(clippy::use_self)] // may be hard to understand what Self even is deep into a function's body
#![allow(clippy::equatable_if_let)] // matches!() adds too much noise for little benefit

pub mod job;

use self::job::JobState;
use fetcher::settings::{
	self,
	context::{Context, StaticContext},
};
use fetcher_config::jobs::{named::JobName, Job};

use color_eyre::Result;
use eframe::NativeOptions;
use egui::{Color32, ScrollArea, SelectableLabel};
use std::{
	collections::{BTreeMap, HashMap},
	path::PathBuf,
};

const COLOR_ERROR: Color32 = Color32::LIGHT_RED;

/// This macro makes the enum contain the requested variant, either by matching and extracting it or by replacing it with a default one
/// Example:
///
/// ```
/// #[derive(Default)]
/// struct First;
///
/// #[derive(Default)]
/// struct Second;
///
/// enum State {
///     First(First),
///     Second(Second),
/// }
///
/// let state = State::First(First);
///
/// // x will remain the old &mut First
/// let x = get_state!(&mut state, State::First);
///
/// // x will become a newly created Second
/// let x = get_state!(&mut state, State::Second);
/// ```
#[macro_export]
macro_rules! get_state {
	($current_state:expr, $desired_state:path) => {{
		let current_state = $current_state;
		match current_state {
			$desired_state(inner) => inner,
			_ => {
				*current_state = $desired_state(Default::default());
				if let $desired_state(state) = current_state {
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
	pub jobs: BTreeMap<JobName, (Job, PathBuf)>,
	pub job_state: HashMap<JobName, JobState>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let context: StaticContext = Box::leak(Box::new(Context {
		data_path: settings::data::default_data_path()?,
		conf_paths: settings::config::default_cfg_dirs()?,
		log_path: settings::log::default_log_path()?,
	}));

	let jobs = settings::config::jobs::get_all(None, context)?
		.into_iter()
		.map(|(job_name, job, path)| (job_name, (job, path)))
		.collect::<BTreeMap<_, (_, _)>>();

	eframe::run_native(
		"Configure fetcher",
		NativeOptions::default(),
		Box::new(|_ctx| {
			Box::new(App {
				current_job: jobs.first_key_value().unwrap().0.clone(),
				jobs,
				job_state: HashMap::default(),
			})
		}),
	)?;

	Ok(())
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
							self.current_job = job_name.clone();
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
					.show(&self.current_job, &mut job.0, ui);
			});
		});
	}
}
