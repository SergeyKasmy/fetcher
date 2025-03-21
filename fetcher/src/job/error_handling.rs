use std::fmt::Write;
use std::{
	ops::ControlFlow,
	time::{Duration, Instant},
};

use tap::TapOptional;
use tokio::time::sleep;

use crate::{error::FetcherError, job::ErrorChainDisplay};

use super::TimePoint;

#[derive(Clone, Debug)]
pub enum ErrorHandling {
	ExponentialBackoffSleep(ExpBackoffSleepState),
	Forward,
	LogAndIgnore,
}

#[derive(Clone, Debug)]
pub struct ExpBackoffSleepState {
	pub max_retries: u32,

	err_count: u32,
	last_error_time: Option<Instant>,
}

impl ErrorHandling {
	pub(super) async fn handle_job_result(
		&mut self,
		res: Result<(), Vec<FetcherError>>,
		job_name: &str,
		job_refresh_time: Option<&TimePoint>,
	) -> ControlFlow<Result<(), Vec<FetcherError>>> {
		let Err(errors) = res else {
			return ControlFlow::Break(Ok(()));
		};

		match self {
			ErrorHandling::Forward => {
				tracing::trace!("Forwarding errors");

				ControlFlow::Break(Err(errors))
			}
			ErrorHandling::LogAndIgnore => {
				for error in &errors {
					tracing::error!("{}", ErrorChainDisplay(error));
				}

				ControlFlow::Continue(())
			}
			ErrorHandling::ExponentialBackoffSleep(state) => {
				handle_errors_exp_backoff(&errors, state, job_name, job_refresh_time)
					.await
					.map_break(|()| Err(errors))
			}
		}
	}
}

impl Default for ErrorHandling {
	fn default() -> Self {
		Self::ExponentialBackoffSleep(Default::default())
	}
}

impl ExpBackoffSleepState {
	const DEFAULT_MAX_RETRY_COUNT: u32 = 15;

	pub fn new() -> Self {
		Self::default()
	}

	pub fn new_with_max_retries(max_retries: u32) -> Self {
		Self {
			max_retries,
			..Self::default()
		}
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

impl Default for ExpBackoffSleepState {
	fn default() -> Self {
		Self {
			max_retries: Self::DEFAULT_MAX_RETRY_COUNT,
			err_count: 0,
			last_error_time: None,
		}
	}
}

async fn handle_errors_exp_backoff(
	errors: &[FetcherError],
	state: &mut ExpBackoffSleepState,
	job_name: &str,
	job_refresh_time: Option<&TimePoint>,
) -> ControlFlow<()> {
	// if time since last error is 2 times longer than the refresh duration, then the error count can safely be reset
	// since there hasn't been any errors for a little while
	// TODO: maybe figure out a more optimal time interval than just 2 times longer than the refresh timer
	if let Some((last_error, refresh_time)) = state.last_error_time.as_ref().zip(job_refresh_time) {
		let last_error_sleep_dur = exponential_backoff_duration(state.err_count);
		match refresh_time {
			TimePoint::Duration(dur) => {
				let twice_refresh_dur = *dur * 2; // two times the refresh duration to make sure the job ran at least twice with no errors
				if last_error.elapsed() > last_error_sleep_dur + twice_refresh_dur {
					state.reset();
				}
			}
			// once a day
			TimePoint::Time(_) => {
				const TWO_DAYS: Duration = Duration::from_secs(
					2 /* days */ * 24 /* hours a day */ * 60 /* mins an hour */ * 60, /* secs a min */
				);

				if last_error.elapsed() > last_error_sleep_dur + TWO_DAYS {
					state.reset();
				}
			}
		}
	}

	// log and filter out network connection errors.
	// they shouldn't be counted against the max error limit because they are ~usually~ temporary and not critical
	let errors_without_net = errors.iter().filter(|e| {
		e.is_connection_error()
			.tap_some(|net_err| {
				tracing::warn!("Network error: {}", ErrorChainDisplay(net_err));
			})
			.is_none()
	});

	if errors_without_net.clone().count() > 0 {
		// max error limit reached
		if state.add_error() {
			tracing::warn!(
				"Maximum error limit reached ({max} out of {max}) for job {job_name}. Stopping retrying...",
				max = state.max_retries
			);
			return ControlFlow::Break(());
		}

		let mut err_msg = format!(
			"Job {job_name} finished {job_err_count} times in an error (out of {max} max allowed)",
			job_err_count = state.err_count,
			max = state.max_retries,
		);

		// log and report all other errors (except for network errors up above)
		for (i, err) in errors_without_net.enumerate() {
			/*
			// FIXME: doesn't and can't even work after the refactor

			if let FetcherError::Transform(transform_err) = &err
				&& let Err(e) = settings::log::log_transform_err(transform_err, job_name)
			{
				tracing::error!("Error logging transform error: {e:?}");
			}
			*/

			_ = write!(
				err_msg,
				"\nError #{err_num}:\n{e}\n",
				err_num = i + 1,
				e = ErrorChainDisplay(err)
			);
		}

		tracing::error!("{}", err_msg);

		/*
		// FIXME: doesn't and can't even work after the refactor

		// TODO: make this a context switch
		// production error reporting
		if !cfg!(debug_assertions) {
			if let Err(e) = report_error(job_name, &err_msg, cx).await {
				tracing::error!("Unable to send error report to the admin: {e:?}",);
			}
		}
		*/
	}

	let sleep_dur = exponential_backoff_duration(state.err_count);
	tracing::info!("Pausing job {job_name} for {}m", sleep_dur.as_secs() / 60);
	sleep(sleep_dur).await;

	ControlFlow::Continue(())
}

/// Sleep in exponentially increasing amount of minutes, beginning with 2^0 = 1 minute.
const fn exponential_backoff_duration(consecutive_err_count: u32) -> Duration {
	// subtract 1 because prev_errors.count() is already set to 1 (because the first error has already happened)
	// but we want to sleep beginning with ^0, not ^1
	let sleep_dur = 2u64.saturating_pow(consecutive_err_count.saturating_sub(1));
	Duration::from_secs(sleep_dur * 60 /* secs in a min */)
}
