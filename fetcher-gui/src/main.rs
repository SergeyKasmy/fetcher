/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod app;

use self::app::App;

use eframe::NativeOptions;

fn main() {
	let job_list = (0..100).map(|i| format!("Job #{i}")).collect::<Vec<_>>();

	eframe::run_native(
		"fetcher",
		NativeOptions::default(),
		Box::new(|_ctx| {
			Box::new(App {
				current_job: job_list[0].clone(),
				job_list,
				..Default::default()
			})
		}),
	)
	.unwrap();
}
