/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`JobResult`] type

use std::{
	any::Any,
	error::Error,
	fmt::{self, Debug},
};

use non_non_full::NonEmptyVec;

use crate::error::FetcherError;

/// A result of a job execution.
///
/// Analogous to [`Result`] but with an added [`JobResult::Panicked`] variant
/// because all panics during a job's execution are caught.
/// See [`Job::run`](`super::Job::run`) for more info.
#[derive(Debug)]
pub enum JobResult {
	/// The job successfully and no tasks returned Err
	Ok,

	/// One or more task returned errors
	Err(NonEmptyVec<FetcherError>),

	/// The job panicked
	Panicked {
		/// Payload of the panic
		payload: Box<dyn Any + Send + 'static>,
	},

	// Note: making this a generic creates way too much noise,
	// especially considering this isn't even used by us
	/// The trigger returned an error
	TriggerFailed(Box<dyn Error + Send + Sync>),
}

impl JobResult {
	/// Unwraps the [`JobResult::Ok`] variant.
	///
	/// # Panics
	/// Panics if the value is [`JobResult::Err`] or [`JobResult::Panicked`].
	pub fn unwrap(self) {
		match self {
			Self::Ok => (),
			Self::Err(errors) => {
				unwrap_failed("called `JobResult::unwrap()` on an `Err` value", &errors);
			}
			Self::Panicked { payload } => unwrap_failed(
				"called `JobResult::unwrap()` on a `Panicked` value",
				&payload,
			),
			Self::TriggerFailed(err) => unwrap_failed(
				"called `JobResult::unwrap()` on a `TriggerFailed` value",
				&err,
			),
		}
	}

	/// Unwraps the [`JobResult::Ok`] variant.
	///
	/// # Panics
	/// Panics if the value is [`JobResult::Err`] or [`JobResult::Panicked`] with a panic message containing the provided message.
	pub fn expect(self, msg: &str) {
		match self {
			Self::Ok => (),
			Self::Err(errors) => unwrap_failed(msg, &errors),
			Self::Panicked { payload } => unwrap_failed(msg, &payload),
			Self::TriggerFailed(err) => unwrap_failed(msg, &err),
		}
	}
}

fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) {
	panic!("{msg}: {error:?}");
}
