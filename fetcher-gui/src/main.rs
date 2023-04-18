/*
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod app;

use self::app::App;
use fetcher_config::jobs::{action::Action, named::JobName, task::Task, Job};

use eframe::NativeOptions;
use std::collections::{BTreeMap, HashMap};

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
						Action::Caps,
						Action::Caps,
						Action::Caps,
						Action::Caps,
						Action::Caps,
						Action::Caps,
						Action::Caps,
						Action::Caps,
						Action::Caps,
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
