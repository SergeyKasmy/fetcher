/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module provides error handling mechanisms for [`Jobs`](`super::Job`), including:
//! - The [`HandleError`] trait used for implementing error handling strategies
//! - Built-in handlers: [`Forward`], [`LogAndIgnore`], and [`ExponentialBackoff`]

mod exponential_backoff;
mod forward;
mod log_and_ignore;

use std::convert::Infallible;
use std::error::Error;

use either::Either;
use non_non_full::NonEmptyVec;

use crate::cancellation_token::CancellationToken;
use crate::error::FetcherError;
use crate::maybe_send::{MaybeSend, MaybeSendSync, MaybeSync};

pub use self::exponential_backoff::ExponentialBackoff;
pub use self::forward::Forward;
pub use self::log_and_ignore::LogAndIgnore;

/// Handler for errors that occur during a [`Job's`](`super::Job`) execution.
pub trait HandleError<Tr>: MaybeSendSync
where
	Tr: MaybeSync,
{
	/// The type of the error that might occure while handling task errors.
	/// Use [`Infallible`](`std::convert::Infallible`) if the handler itself never errors
	type HandlerErr: Error;

	/// This function will be called each time an error occures during the execution of a [`Job`](`super::Job`).
	///
	/// The error handler decides what should happen with the job afterwards via the [`HandleErrorResult`] type.
	fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_, Tr>,
	) -> impl Future<Output = HandleErrorResult<Self::HandlerErr>> + MaybeSend;
}

/// Context about the parent [`Job`](`super::Job`) provided to a [`HandleError`]
pub struct HandleErrorContext<'a, Tr>
where
	Tr: MaybeSync,
{
	/// Name of the Job
	pub job_name: &'a str,

	/// The job's [`Job::trigger`](`super::Job::trigger`)
	pub job_trigger: &'a Tr,

	/// The job's [`Job::cancel_token`](`super::Job::cancel_token`)
	pub cancel_token: Option<&'a mut CancellationToken>,
}

/// What should happen after the handler returns
pub enum HandleErrorResult<E> {
	/// The [`Job`](`super::Job`) should be resumed as if nothing happened
	ResumeJob {
		/// Should the job call [`Trigger::wait`](`super::Trigger`) afterwards or should it just restart the job from the beginning?
		///
		/// Error handlers that sleep (e.g. [`ExponentialBackoff`]) should return `false` to avoid double-sleep.\
		/// Error handlers that don't actually handle errors, e.g. just log them (e.g. [`LogAndIgnore`]) should return `true`
		/// to make sure the job sleeps between invokations.
		wait_for_trigger: bool,
	},

	/// The [`Job`](`super::Job`) should be stopped and these errors should be returned
	StopWithErrors(NonEmptyVec<FetcherError>),

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
			HandleErrorResult::ResumeJob {
				wait_for_trigger: wait_on_the_trigger,
			} => HandleErrorResult::ResumeJob {
				wait_for_trigger: wait_on_the_trigger,
			},
			HandleErrorResult::StopWithErrors(e) => HandleErrorResult::StopWithErrors(e),
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

impl<A, B, Tr> HandleError<Tr> for Either<A, B>
where
	A: HandleError<Tr>,
	B: HandleError<Tr>,
	Tr: MaybeSync,
{
	type HandlerErr = Either<A::HandlerErr, B::HandlerErr>;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_, Tr>,
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
impl<Tr> HandleError<Tr> for ()
where
	Tr: MaybeSync,
{
	type HandlerErr = <Forward as HandleError<Tr>>::HandlerErr;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_, Tr>,
	) -> HandleErrorResult<Self::HandlerErr> {
		Forward.handle_errors(errors, cx).await
	}
}

impl<Tr> HandleError<Tr> for Infallible
where
	Tr: MaybeSync,
{
	type HandlerErr = Infallible;

	async fn handle_errors(
		&mut self,
		_errors: NonEmptyVec<FetcherError>,
		_cx: HandleErrorContext<'_, Tr>,
	) -> HandleErrorResult<Self::HandlerErr> {
		match *self {}
	}
}

#[cfg(feature = "nightly")]
impl<Tr> HandleError<Tr> for !
where
	Tr: MaybeSync,
{
	type HandlerErr = !;

	async fn handle_errors(
		&mut self,
		_errors: NonEmptyVec<FetcherError>,
		_cx: HandleErrorContext<'_, Tr>,
	) -> HandleErrorResult<Self::HandlerErr> {
		match *self {}
	}
}

impl<H, Tr> HandleError<Tr> for Option<H>
where
	H: HandleError<Tr>,
	Tr: MaybeSync,
{
	type HandlerErr = H::HandlerErr;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_, Tr>,
	) -> HandleErrorResult<Self::HandlerErr> {
		let Some(inner) = self else {
			match Forward.handle_errors(errors, cx).await {
				HandleErrorResult::ResumeJob {
					wait_for_trigger: wait_on_the_trigger,
				} => {
					return HandleErrorResult::ResumeJob {
						wait_for_trigger: wait_on_the_trigger,
					};
				}
				HandleErrorResult::StopWithErrors(e) => {
					return HandleErrorResult::StopWithErrors(e);
				}
				HandleErrorResult::ErrWhileHandling { err, .. } => match err {},
			}
		};

		inner.handle_errors(errors, cx).await
	}
}

impl<H, Tr> HandleError<Tr> for &mut H
where
	H: HandleError<Tr>,
	Tr: MaybeSync,
{
	type HandlerErr = H::HandlerErr;

	fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		cx: HandleErrorContext<'_, Tr>,
	) -> impl Future<Output = HandleErrorResult<Self::HandlerErr>> + MaybeSend {
		(*self).handle_errors(errors, cx)
	}
}
