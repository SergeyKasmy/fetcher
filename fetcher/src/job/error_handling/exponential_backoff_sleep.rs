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

#[derive(Clone, Debug)]
pub struct ExponentialBackoffSleep {
	// TODO: make a const generic?
	pub max_retries: u32,

	err_count: u32,
	last_error_time: Option<Instant>,
}

impl HandleError for ExponentialBackoffSleep {
	type Err = Infallible;

	async fn handle_errors(
		&mut self,
		errors: Vec<FetcherError>,
		cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::Err> {
		match self.should_continue(&errors, cx).await {
			ExpBackoffHandleErrorResult::ContinueTheJob => HandleErrorResult::ContinueJob,
			ExpBackoffHandleErrorResult::ReturnTheErrors => {
				HandleErrorResult::StopAndReturnErrs(errors)
			}
		}
	}
}

enum ExpBackoffHandleErrorResult {
	ContinueTheJob,
	ReturnTheErrors,
}

impl ExponentialBackoffSleep {
	const DEFAULT_MAX_RETRY_COUNT: u32 = 15;

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

		match self.add_and_log_fatal_errors(errors_without_net, cx.job_name) {
			ExpBackoffHandleErrorResult::ReturnTheErrors => {
				return ExpBackoffHandleErrorResult::ReturnTheErrors;
			}
			// continue the job after the pause
			ExpBackoffHandleErrorResult::ContinueTheJob => (),
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
	let sleep_dur = 2u64.saturating_pow(consecutive_err_count.saturating_sub(1));
	Duration::from_secs(sleep_dur * 60 /* secs in a min */)
}
