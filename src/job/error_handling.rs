/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module provides error handling mechanisms for [`Jobs`](`super::Job`), including:
//! - The [`HandleError`] trait used for implementing error handling strategies
//! - Built-in handlers: [`Forward`], [`LogAndIgnore`], and [`ExponentialBackoff`]

// TODO: add tests

mod exponential_backoff;
mod forward;
mod log_and_ignore;

use std::convert::Infallible;
use std::error::Error;

use either::Either;
use non_non_full::NonEmptyVec;

use crate::ctrl_c_signal::CtrlCSignalChannel;
use crate::error::FetcherError;
use crate::maybe_send::{MaybeSend, MaybeSendSync};

use super::RefreshTime;

pub use self::exponential_backoff::ExponentialBackoff;
pub use self::forward::Forward;
pub use self::log_and_ignore::LogAndIgnore;

/// Handler for errors that occur during a [`Job's`](`super::Job`) execution.
pub trait HandleError: MaybeSendSync {
	/// The type of the error that might occure while handling task errors.
	/// Use [`Infallible`](`std::convert::Infallible`) if the handler itself never errors
	type HandlerErr: Error;

	/// This function will be called each time an error occures during the execution of a [`Job`](`super::Job`).
	///
	/// The error handler decides what should happen with the job afterwards via the [`HandleErrorResult`] type.
	fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_>,
	) -> impl Future<Output = HandleErrorResult<Self::HandlerErr>> + MaybeSend;
}

/// Context about the parent [`Job`](`super::Job`) provided to a [`HandleError`]
pub struct HandleErrorContext<'a> {
	/// Name of the Job
	pub job_name: &'a str,

	/// The job's [`Job::refresh_time`](`super::Job::refresh_time`)
	pub job_refresh_time: &'a RefreshTime,

	/// The job's [`Job::ctrlc_chan`](`super::Job::ctrlc_chan`)
	pub ctrlc_chan: Option<&'a mut CtrlCSignalChannel>,
}

/// What should happen after the handler returns
pub enum HandleErrorResult<E> {
	/// The [`Job`](`super::Job`) should be continued as if nothing happened
	ContinueJob,

	/// The [`Job`](`super::Job`) should be stopped and these errors should be returned
	StopAndReturnErrs(NonEmptyVec<FetcherError>),

	/// The [`Job`](`super::Job`) should be stopped because an error has occured while handling the errors
	ErrWhileHandling {
		/// The error that occured during executing of the error handler
		err: E,

		/// The original errors that caused the error handler to be invoked in the first place
		original_errors: NonEmptyVec<FetcherError>,
	},
}

impl<E> HandleErrorResult<E> {
	/// Maps the handler error [`HandleErrorResult::ErrWhileHandling::err`] by applying the provided funtion to it.
	pub fn map_handler_err<U, F>(self, f: F) -> HandleErrorResult<U>
	where
		F: FnOnce(E) -> U,
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

impl<A, B> HandleError for Either<A, B>
where
	A: HandleError,
	B: HandleError,
{
	type HandlerErr = Either<A::HandlerErr, B::HandlerErr>;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::HandlerErr> {
		match self {
			Either::Left(a) => a
				.handle_errors(errors, cx)
				.await
				.map_handler_err(Either::Left),

			Either::Right(b) => b
				.handle_errors(errors, cx)
				.await
				.map_handler_err(Either::Right),
		}
	}
}

/// The same as [`Forward`]
impl HandleError for () {
	type HandlerErr = <Forward as HandleError>::HandlerErr;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::HandlerErr> {
		Forward.handle_errors(errors, cx).await
	}
}

impl HandleError for Infallible {
	type HandlerErr = Infallible;

	async fn handle_errors(
		&mut self,
		_errors: NonEmptyVec<FetcherError>,
		_cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::HandlerErr> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl HandleError for ! {
	type HandlerErr = !;

	async fn handle_errors(
		&mut self,
		_errors: NonEmptyVec<FetcherError>,
		_cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::HandlerErr> {
		match *self {}
	}
}

impl<H> HandleError for Option<H>
where
	H: HandleError,
{
	type HandlerErr = H::HandlerErr;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::HandlerErr> {
		let Some(inner) = self else {
			match Forward.handle_errors(errors, cx).await {
				HandleErrorResult::ContinueJob => return HandleErrorResult::ContinueJob,
				HandleErrorResult::StopAndReturnErrs(e) => {
					return HandleErrorResult::StopAndReturnErrs(e);
				}
				HandleErrorResult::ErrWhileHandling { err, .. } => match err {},
			}
		};

		inner.handle_errors(errors, cx).await
	}
}
