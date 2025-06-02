/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`ExponentialBackoff`] error handler

use std::{
	convert::Infallible,
	fmt::Write,
	time::{Duration, Instant},
};

use non_non_full::NonEmptyVec;
use rand::Rng;
use tap::TapOptional;
use tokio::{select, time::sleep};

use crate::{
	error::FetcherError,
	job::{ErrorChainDisplay, Trigger, cancel_wait},
	maybe_send::MaybeSync,
};

use super::{HandleError, HandleErrorContext, HandleErrorResult};

/// An error handler that implements exponential backoff retry logic.
///
/// When errors occur during job execution, this handler will:
/// 1. Log any network errors and ignore them
/// 2. Pause the job for an exponentially increasing duration between retries
/// 3. Track consecutive errors and stop retrying after reaching [`ExponentialBackoff::max_attempts`]
///
/// The pause duration starts at 1 minute (2^0) and doubles with each consecutive error:
/// - 1st error: 1 minute
/// - 2nd error: 2 minutes
/// - 3rd error: 4 minutes
/// - 4th error: 8 minutes
///   And so on...
///
/// The error count is reset if the job runs successfully for:
/// - Interval-based jobs: Two successful intervals without errors
/// - Time-based jobs: Two days without errors
///
/// # Example
/// ```rust
/// use fetcher::job::error_handling::ExponentialBackoff;
///
/// let mut handler = ExponentialBackoff::new(); // Uses default max_retries of 15
/// // Or configure custom attempt limit:
/// handler.max_attempts = 5;
/// ```
#[derive(Clone, Debug)]
pub struct ExponentialBackoff {
	/// Maximum number of consecutive errors allowed before giving up.
	/// Defaults to [`DEFAULT_MAX_ATTEMPT_COUNT`](`Self::DEFAULT_MAX_ATTEMPT_COUNT`) (15).
	pub max_attempts: u32,

	/// Use jitter when calculating pause duration
	pub use_jitter: bool,

	/// How much to pause the job for when only network errors happened (e.g. internet got disconnected)
	///
	/// Note: This is contant and doesn't increate current attempt count
	/// Defaults to [`DEFAULT_NETWORK_ERROR_PAUSE_DURATION`](`Self::DEFAULT_NETWORK_ERROR_PAUSE_DURATION`) (5 minutes).
	pub pause_duration_net_error: Duration,

	/// Infomation about the last error
	last_error_info: Option<ErrorInfo>,
}

#[derive(Clone, Debug)]
struct ErrorInfo {
	attempt: u32,
	happened_at: Instant,
	must_sleep_for: Duration,
}

impl<Tr> HandleError<Tr> for ExponentialBackoff
where
	Tr: Trigger,
{
	type HandlerErr = Infallible;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_, Tr>,
	) -> HandleErrorResult<Self::HandlerErr> {
		if self.should_continue(&errors, cx).await {
			HandleErrorResult::ContinueJob
		} else {
			HandleErrorResult::StopAndReturnErrs(errors)
		}
	}
}

impl ExponentialBackoff {
	#[expect(missing_docs, reason = "self-explanatory")]
	pub const DEFAULT_MAX_ATTEMPT_COUNT: u32 = 15;
	#[expect(missing_docs, reason = "self-explanatory")]
	pub const DEFAULT_NETWORK_ERROR_PAUSE_DURATION: Duration =
		Duration::from_secs(5 * 60 /* secs in a min*/);

	/// Creates a new [`ExponentialBackoff`] instance with the default values.
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new [`ExponentialBackoff`] instance with the max attempt count set to `max_attempts`.
	#[must_use]
	pub fn new_with_conf(
		max_attempts: u32,
		use_jitter: bool,
		pause_duration_net_error: Duration,
	) -> Self {
		Self {
			max_attempts,
			use_jitter,
			pause_duration_net_error,
			..Default::default()
		}
	}

	/// Gets the number of the next attempt, the attempt the handler would handle if it was called right away.
	///
	/// Can be used to wrap the [`ExponentialBackoff`] error handler with additional logging or notifications
	///
	/// # Returns
	/// What attempt the next immediate call to [`ExponentialBackoff::handle_errors`] would be,
	///
	/// # Note
	/// If this count reaches [`ExponentialBackoff::max_attempts`], then the next call to [`ExponentialBackoff::handle_errors`]
	/// will actually just stop the job completely and returns the errors back.
	#[must_use]
	pub fn next_attempt<Tr: Trigger>(&mut self, job_trigger: &Tr) -> u32 {
		self.reset_error_count(job_trigger);

		match self.check_limit_reached() {
			AttemptLimitReached::No { current_attempt } => current_attempt,
			AttemptLimitReached::Yes => self.max_attempts,
		}
	}
}

#[derive(Clone, Copy, Debug)]
enum AttemptLimitReached {
	No { current_attempt: u32 },
	Yes,
}

impl ExponentialBackoff {
	/// Returns `true` if the job should continue executing.
	/// Returns `false` if the job should stop.
	async fn should_continue<Tr: Trigger>(
		&mut self,
		errors: &[FetcherError],
		cx: HandleErrorContext<'_, Tr>,
	) -> bool {
		// reset counter if a while has passed since last error
		self.reset_error_count(cx.job_trigger);

		// get all errors that are not network related
		let fatal_errors = errors.iter().filter(|e| {
			e.is_connection_error()
				.tap_some(|net_err| {
					tracing::warn!("Network error: {}", ErrorChainDisplay(net_err));
				})
				.is_none()
		});

		// if all errors are network related(e.g. internet disconnected), pause for a static amount of time and try again
		if fatal_errors.clone().count() == 0 {
			return pause_job(self.pause_duration_net_error, cx).await;
		}

		// check if the attempt limit has been reached
		let attempt_limit_reached = self.check_limit_reached();

		// log all fatal errors that have happened
		self.log(attempt_limit_reached, fatal_errors, cx.job_name);

		let AttemptLimitReached::No { current_attempt } = attempt_limit_reached else {
			// limit reached, stop the job
			return false;
		};

		let pause_duration =
			exponential_backoff_duration(current_attempt, self.use_jitter, rand::rng());

		self.last_error_info = Some(ErrorInfo {
			attempt: current_attempt,
			happened_at: Instant::now(),
			must_sleep_for: pause_duration,
		});

		pause_job(pause_duration, cx).await
	}

	// TODO: resets too early if after a long pause internet got disconnected for a while.
	// Maybe make network errors reset last_error.happened_at?
	/// Resets the attempt counter if enough time has passed since last error
	fn reset_error_count<Tr: Trigger>(&mut self, job_trigger: &Tr) {
		let Some(last_error) = self.last_error_info.as_ref() else {
			return;
		};

		if last_error.happened_at.elapsed()
			> last_error.must_sleep_for + job_trigger.twice_as_duration()
		{
			self.reset();
		}
	}

	/// Returns `None` if the max attempt limit has been reached.
	/// Otherwise reaturns `Some` containing the current attempt number.
	fn check_limit_reached(&self) -> AttemptLimitReached {
		let prev_attempt = self
			.last_error_info
			.as_ref()
			.map(|info| info.attempt)
			.unwrap_or(0);

		let current_attempt = prev_attempt + 1;

		if current_attempt >= self.max_attempts {
			AttemptLimitReached::Yes
		} else {
			AttemptLimitReached::No { current_attempt }
		}
	}

	fn log<'a>(
		&self,
		attempt: AttemptLimitReached,
		fatal_errors: impl Iterator<Item = &'a FetcherError>,
		job_name: &str,
	) {
		let current_attempt = match attempt {
			AttemptLimitReached::Yes => {
				tracing::warn!(
					"Maximum error limit reached ({max} out of {max}) for job {job_name}. Stopping retrying...",
					max = self.max_attempts,
				);

				return;
			}
			AttemptLimitReached::No { current_attempt } => current_attempt,
		};

		let mut err_msg = format!(
			"Job {job_name} finished {current_attempt}/{max} times in an error ",
			max = self.max_attempts,
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
	}

	fn reset(&mut self) {
		self.last_error_info = None;
	}
}

impl Default for ExponentialBackoff {
	fn default() -> Self {
		Self {
			max_attempts: Self::DEFAULT_MAX_ATTEMPT_COUNT,
			use_jitter: true,
			pause_duration_net_error: Self::DEFAULT_NETWORK_ERROR_PAUSE_DURATION,
			last_error_info: None,
		}
	}
}

// TODO: add base delay if users need it (together with max delay)
/// Sleep in exponentially increasing amount of minutes, beginning with 2^0 = 1 minute.
fn exponential_backoff_duration(attempt: u32, use_jitter: bool, mut rng: impl Rng) -> Duration {
	// 1 attempt -> wait 2^0=1 mins
	// 2 attempt -> wait 2^1=2 mins
	// 3 attempt -> wait 2^2=4 mins
	// 4 attempt -> wait 2^3=8 mins
	let base_duration_min = 2u64.saturating_pow(attempt.saturating_sub(1));
	let base_duration_sec = base_duration_min * 60;

	let final_duration = if use_jitter {
		#[expect(clippy::cast_precision_loss, reason = "what other way is there?")]
		let duration_secs_f64 = base_duration_sec as f64 * (rng.random::<f64>() + 0.5);

		#[expect(clippy::cast_possible_truncation, reason = "what other way is there?")]
		#[expect(clippy::cast_sign_loss, reason = "always positive")]
		let duration_secs = duration_secs_f64.round() as u64;

		tracing::debug!(
			"Calculated exponential backoff duration: base = {base_duration_min}m ({base_duration_sec}s), with jitter = {duration_secs_f64}s (rounded to {duration_secs}s, ~{}m)",
			duration_secs / 60
		);

		duration_secs
	} else {
		tracing::debug!("Calculated exponential backoff duration: {base_duration_min}m");

		base_duration_sec
	};

	Duration::from_secs(final_duration)
}

/// Returns `true` if the job should continue after the pause.
/// Returns `false` if the pause was interrupted and the job should stop
async fn pause_job<Tr: MaybeSync>(dur: Duration, cx: HandleErrorContext<'_, Tr>) -> bool {
	tracing::info!("Pausing job {} for {}m", cx.job_name, dur.as_secs() / 60);

	select! {
		() = sleep(dur) => {
			true
		}
		() = cancel_wait(cx.cancel_token) => {
			tracing::debug!("Job terminated mid exponential backoff pause");
			false
		}
	}
}

#[cfg(test)]
mod tests {
	#![expect(clippy::cast_precision_loss, clippy::unimplemented)]

	use std::time::Duration;

	use rand::Rng;

	use super::exponential_backoff_duration;

	/// Asserts that [`exponential_backoff_duration`] returned the expected duration (of within range if jitter is enabled)
	fn check_exp_backoff_duration(
		attempt: u32,
		use_jitter: bool,
		expected_result: Duration,
		rng: impl Rng,
	) {
		let dur = exponential_backoff_duration(attempt, use_jitter, rng);
		if use_jitter {
			let dur = dur.as_secs() as f64;
			let expected_dur = expected_result.as_secs() as f64;
			assert!(dur <= (expected_dur * 1.5) && dur >= (expected_dur / 2.0));
		} else {
			assert_eq!(dur, expected_result, "attempt: {attempt}");
		}
	}

	/// Converts minutes into a [`Duration`]
	fn m(mins: u64) -> Duration {
		Duration::from_secs(mins * 60 /* secs in a min*/)
	}

	#[test]
	fn exponential_backoff_duration_no_jitter() {
		for i in 0u32..=15 {
			let expected_mins = 2u64.pow(i.saturating_sub(1));
			check_exp_backoff_duration(i, false, m(expected_mins), rand::rng());
		}
	}

	#[test]
	fn exponential_backoff_duration_with_jitter() {
		for i in 0u32..=15 {
			let expected_mins = 2u64.pow(i.saturating_sub(1));
			check_exp_backoff_duration(i, true, m(expected_mins), rand::rng());
		}
	}

	#[test]
	fn exponential_backoff_duration_with_fake_jitter() {
		/// An Rng source that alternates between the MIN and MAX of a type
		struct AlwaysExtremes(bool);

		impl rand::RngCore for AlwaysExtremes {
			fn next_u64(&mut self) -> u64 {
				let min_or_max = self.0;
				self.0 = !self.0;
				if min_or_max { u64::MIN } else { u64::MAX }
			}

			fn next_u32(&mut self) -> u32 {
				unimplemented!()
			}

			fn fill_bytes(&mut self, _dst: &mut [u8]) {
				unimplemented!()
			}
		}

		let mut rng = AlwaysExtremes(true);

		for i in 0u32..=15 {
			let expected_mins = 2u64.pow(i.saturating_sub(1));
			// two separate calls will generate two jitter values in the two extremes of the allowed range. Confirm both are within range
			check_exp_backoff_duration(i, true, m(expected_mins), &mut rng);
			check_exp_backoff_duration(i, true, m(expected_mins), &mut rng);
		}
	}
}
