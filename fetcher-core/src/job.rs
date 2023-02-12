/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Job`] struct and the entryway to the library

use futures::future::join_all;
use std::time::Duration;
use tokio::time::sleep;

use crate::{
	error::{Error, ErrorChainExt},
	task::Task,
};

/// A single job, containing a single or a couple [`tasks`](`Task`), possibly refetching every set amount of time
#[derive(Debug)]
pub struct Job {
	/// The tasks to run
	pub tasks: Vec<Task>,

	/// Refetch every interval or just run once
	pub refetch_interval: Option<Duration>,
}

impl Job {
	/// Run this job to completion or return early on an error
	///
	/// # Errors
	/// if any of the inner tasks return an error, refer to [`Task`] documentation
	pub async fn run(&mut self, err_max_count: u32) -> Result<(), Error> {
		// exit with an error if there were too many consecutive errors
		// let err_max_count: u32 = 15; // around 22 days max pause time

		// number of consecutive(!!!) errors.
		// we tolerate a pretty big amount for various reasons (being rate limited, server error, etc) but not infinite
		let mut err_count = 0;

		loop {
			let jobs = self.tasks.iter_mut().map(Task::run);
			let results = join_all(jobs).await;

			let mut all_ok = true;
			for res in results {
				match res {
					Ok(()) => (),
					Err(err) => {
						if err_count == err_max_count {
							return Err(err);
						}

						all_ok = false;

						if let Some(network_err) = err.is_connection_error() {
							tracing::warn!("Network error: {}", network_err.display_chain());
						} else {
							// TODO
							// if let Error::Transform(transform_err) = &err {
							// 	settings::log::log_transform_err(transform_err, &job_name)?;
							// }

							let err_msg = format!(
								"Error #{} out of {} max allowed:\n{}",
								err_count + 1, // +1 cause we are counting from 0 but it'd be strange to show "Error (0 out of 255)" to users
								err_max_count + 1,
								err.display_chain()
							);
							tracing::error!("{}", err_msg);

							// sleep in exponention amount of minutes, begginning with 2^0 = 1 minute
							let sleep_dur = 2u64.saturating_pow(err_count);
							// tracing::info!("Pausing task {job_name} for {sleep_dur}m");

							sleep(Duration::from_secs(sleep_dur * 60 /* secs in a min*/)).await;
							err_count += 1;

							// TODO: make this a context switch
							// production error reporting
							// if !cfg!(debug_assertions) {
							// 	if let Err(e) = report_error(&job_name, &err_msg, cx).await {
							// 		tracing::error!(
							// 			"Unable to send error report to the admin: {e:?}",
							// 		);
							// 	}
							// }
						}
					}
				}
			}
			if all_ok {
				err_count = 0;
			}

			match self.refetch_interval {
				Some(refetch_interval) => {
					tracing::debug!("Putting job to sleep for {}m", refetch_interval.as_secs());
					sleep(refetch_interval).await;
				}
				None => break,
			}
		}

		Ok(())
	}
}
