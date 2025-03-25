mod exponential_backoff_sleep;

use std::convert::Infallible;

use either::Either;

use crate::ctrl_c_signal::CtrlCSignalChannel;
use crate::{error::FetcherError, job::ErrorChainDisplay};

use super::TimePoint;

pub use self::exponential_backoff_sleep::ExponentialBackoffSleep;

pub trait HandleError {
	type Err: Into<FetcherError>;

	async fn handle_errors(
		&mut self,
		errors: Vec<FetcherError>,
		cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::Err>;
}

pub struct HandleErrorContext<'a> {
	pub job_name: &'a str,
	pub job_refresh_time: Option<&'a TimePoint>,
	pub ctrlc_chan: Option<&'a mut CtrlCSignalChannel>,
}

pub enum HandleErrorResult<E> {
	ContinueJob,
	StopAndReturnErrs(Vec<FetcherError>),
	ErrWhileHandling {
		err: E,
		original_errors: Vec<FetcherError>,
	},
}

pub struct Forward;
pub struct LogAndIgnore;

impl HandleError for Forward {
	type Err = Infallible;

	async fn handle_errors(
		&mut self,
		errors: Vec<FetcherError>,
		_cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::Err> {
		tracing::trace!("Forwarding errors");

		HandleErrorResult::StopAndReturnErrs(errors)
	}
}

impl HandleError for LogAndIgnore {
	type Err = Infallible;

	async fn handle_errors(
		&mut self,
		errors: Vec<FetcherError>,
		_cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::Err> {
		for error in &errors {
			tracing::error!("{}", ErrorChainDisplay(error));
		}

		HandleErrorResult::ContinueJob
	}
}

impl<A, B> HandleError for Either<A, B>
where
	A: HandleError,
	B: HandleError,
{
	type Err = Either<A::Err, B::Err>;

	async fn handle_errors(
		&mut self,
		errors: Vec<FetcherError>,
		cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::Err> {
		match self {
			Either::Left(a) => a
				.handle_errors(errors, cx)
				.await
				.map_original_err(Either::Left),

			Either::Right(b) => b
				.handle_errors(errors, cx)
				.await
				.map_original_err(Either::Right),
		}
	}
}

impl<T> HandleErrorResult<T> {
	pub fn map_original_err<U, F>(self, f: F) -> HandleErrorResult<U>
	where
		F: FnOnce(T) -> U,
	{
		match self {
			HandleErrorResult::ContinueJob => HandleErrorResult::ContinueJob,
			HandleErrorResult::StopAndReturnErrs(e) => HandleErrorResult::StopAndReturnErrs(e),
			HandleErrorResult::ErrWhileHandling {
				err,
				original_errors,
			} => HandleErrorResult::ErrWhileHandling {
				err: f(err),
				original_errors,
			},
		}
	}
}
