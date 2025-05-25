/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ExponentialBackoffSleep`] error handler

use std::{
	convert::Infallible,
	fmt::Write,
	time::{Duration, Instant},
};

use tap::TapOptional;
use tokio::{select, time::sleep};

use crate::{
	error::FetcherError,
	job::{ErrorChainDisplay, TimePoint, ctrlc_signaled},
};

use super::{HandleError, HandleErrorContext, HandleErrorResult};

/// An error handler that implements exponential backoff retry logic.
///
/// When errors occur during job execution, this handler will:
/// 1. Log any network errors and ignore them
/// 2. Pause the job for an exponentially increasing duration between retries
/// 3. Track consecutive errors and stop retrying after reaching [`ExponentialBackoffSleep::max_retries`]
///
/// The sleep duration starts at 1 minute (2^0) and doubles with each consecutive error:
/// - 1st error: 1 minute
/// - 2nd error: 2 minutes
/// - 3rd error: 4 minutes
/// - 4th error: 8 minutes  
/// And so on...
///
/// The error count is reset if the job runs successfully for:
/// - Interval-based jobs: Two successful intervals without errors
/// - Time-based jobs: Two days without errors
///
/// # Example
/// ```rust
/// use fetcher::job::error_handling::ExponentialBackoffSleep;
///
/// let mut handler = ExponentialBackoffSleep::default(); // Uses default max_retries of 15
/// // Or configure custom retry limit:
/// handler.max_retries = 5;
/// ```
#[derive(Clone, Debug)]
pub struct ExponentialBackoffSleep {
	/// Maximum number of consecutive errors allowed before stopping retries.
	/// Defaults to [`DEFAULT_MAX_RETRY_COUNT`](`Self::DEFAULT_MAX_RETRY_COUNT`) (15).
	pub max_retries: u32,

	err_count: u32,
	last_error_time: Option<Instant>,
}

enum ExpBackoffHandleErrorResult {
	ContinueTheJob,
	ReturnTheErrors,
}

impl HandleError for ExponentialBackoffSleep {
	type HandlerErr = Infallible;

	async fn handle_errors(
		&mut self,
		errors: Vec<FetcherError>,
		cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::HandlerErr> {
		match self.should_continue(&errors, cx).await {
			ExpBackoffHandleErrorResult::ContinueTheJob => HandleErrorResult::ContinueJob,
			ExpBackoffHandleErrorResult::ReturnTheErrors => {
				HandleErrorResult::StopAndReturnErrs(errors)
			}
		}
	}
}

// TODO: add a new method
impl ExponentialBackoffSleep {
	#[expect(missing_docs, reason = "self-explanatory")]
	pub const DEFAULT_MAX_RETRY_COUNT: u32 = 15;

	/// Creates a new [`ExponentialBackoffSleep`] instance with the default max retry count.
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new [`ExponentialBackoffSleep`] instance with the max retry count set to `max_retries`.
	pub fn new_with_max_retries(max_retries: u32) -> Self {
		Self {
			max_retries,
			..Default::default()
		}
	}

	// TODO: add example
	/// Returns the current count of consecutive errors.
	///
	/// If this count reaches [`ExponentialBackoffSleep::max_retries`], then the job completely stops and returns the error.
	#[must_use]
	pub fn err_count(&self) -> u32 {
		self.err_count
	}

	async fn should_continue(
		&mut self,
		errors: &[FetcherError],
		cx: HandleErrorContext<'_>,
	) -> ExpBackoffHandleErrorResult {
		self.reset_error_count(cx.job_refresh_time);

		let errors_without_net = errors.iter().filter(|e| {
			e.is_connection_error()
				.tap_some(|net_err| {
					tracing::warn!("Network error: {}", ErrorChainDisplay(net_err));
				})
				.is_none()
		});

		if errors_without_net.clone().count() > 0 {
			match self.add_and_log_fatal_errors(errors_without_net, cx.job_name) {
				ExpBackoffHandleErrorResult::ReturnTheErrors => {
					return ExpBackoffHandleErrorResult::ReturnTheErrors;
				}
				// continue the job after the pause
				ExpBackoffHandleErrorResult::ContinueTheJob => (),
			}
		}

		let sleep_dur = exponential_backoff_duration(self.err_count);
		tracing::info!(
			"Pausing job {} for {}m",
			cx.job_name,
			sleep_dur.as_secs() / 60
		);

		select! {
			() = sleep(sleep_dur) => {
				ExpBackoffHandleErrorResult::ContinueTheJob
			}
			() = ctrlc_signaled(cx.ctrlc_chan) => {
				ExpBackoffHandleErrorResult::ReturnTheErrors
			}
		}
	}

	/// Resets the consecutive error counter if enough time has passed
	fn reset_error_count(&mut self, job_refresh_time: Option<&TimePoint>) {
		let Some((last_error, refresh_time)) = self.last_error_time.as_ref().zip(job_refresh_time)
		else {
			return;
		};

		let last_error_sleep_dur = exponential_backoff_duration(self.err_count);
		match refresh_time {
			TimePoint::Duration(dur) => {
				let twice_refresh_dur = *dur * 2; // two times the refresh duration to make sure the job ran at least twice with no errors
				if last_error.elapsed() > last_error_sleep_dur + twice_refresh_dur {
					self.reset();
				}
			}
			// once a day
			TimePoint::Time(_) => {
				const TWO_DAYS: Duration = Duration::from_secs(
					2 /* days */ * 24 /* hours a day */ * 60 /* mins an hour */ * 60, /* secs a min */
				);

				if last_error.elapsed() > last_error_sleep_dur + TWO_DAYS {
					self.reset();
				}
			}
		}
	}

	fn add_and_log_fatal_errors<'a>(
		&mut self,
		fatal_errors: impl Iterator<Item = &'a FetcherError>,
		job_name: &str,
	) -> ExpBackoffHandleErrorResult {
		// max error limit reached
		if self.add_error() {
			tracing::warn!(
				"Maximum error limit reached ({max} out of {max}) for job {job_name}. Stopping retrying...",
				max = self.max_retries,
			);
			return ExpBackoffHandleErrorResult::ReturnTheErrors;
		}

		let mut err_msg = format!(
			"Job {job_name} finished {job_err_count} times in an error (out of {max} max allowed)",
			job_err_count = self.err_count,
			max = self.max_retries,
		);

		// log and report all other errors (except for network errors up above)
		for (i, err) in fatal_errors.enumerate() {
			_ = write!(
				err_msg,
				"\nError #{err_num}:\n{e}\n",
				err_num = i + 1,
				e = ErrorChainDisplay(err)
			);
		}

		tracing::error!("{}", err_msg);

		ExpBackoffHandleErrorResult::ContinueTheJob
	}

	/// Returns true if max limit is reached
	fn add_error(&mut self) -> bool {
		self.err_count += 1;

		if self.err_count >= self.max_retries {
			return true;
		}

		self.last_error_time = Some(Instant::now());

		false
	}

	fn reset(&mut self) {
		self.err_count = 0;
		self.last_error_time = None;
	}
}

impl Default for ExponentialBackoffSleep {
	fn default() -> Self {
		Self {
			max_retries: Self::DEFAULT_MAX_RETRY_COUNT,
			err_count: 0,
			last_error_time: None,
		}
	}
}

/// Sleep in exponentially increasing amount of minutes, beginning with 2^0 = 1 minute.
const fn exponential_backoff_duration(consecutive_err_count: u32) -> Duration {
	// subtract 1 because prev_errors.count() is already set to 1 (because the first error has already happened)
	// but we want to sleep beginning with ^0, not ^1
	// FIXME: add random jitter. Maybe move to a battle-tested implementation instead? (e.g. backoff crate)
	let dur = 2u64.saturating_pow(consecutive_err_count.saturating_sub(1));
	Duration::from_secs(dur * 60 /* secs in a min */)
}
