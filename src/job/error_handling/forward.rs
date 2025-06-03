/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Forward`] error handler

use std::convert::Infallible;

use non_non_full::NonEmptyVec;

use super::{HandleError, HandleErrorContext, HandleErrorResult};
use crate::{error::FetcherError, maybe_send::MaybeSync};

/// Error handler that forwards all errors to the caller, stopping the job immediately.
#[derive(Clone, Copy, Debug)]
pub struct Forward;

impl<Tr> HandleError<Tr> for Forward
where
	Tr: MaybeSync,
{
	type HandlerErr = Infallible;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		_cx: HandleErrorContext<'_, Tr>,
	) -> HandleErrorResult<Self::HandlerErr> {
		tracing::trace!("Forwarding errors");

		HandleErrorResult::StopWithErrors(errors)
	}
}

#[cfg(test)]
mod tests {
	use std::{error::Error, io};

	use assert_matches::assert_matches;

	use crate::{
		Job,
		actions::{
			transform_fn,
			transforms::error::{TransformError, TransformErrorKind},
		},
		entry::Entry,
		error::FetcherError,
		job::JobResult,
	};

	use super::Forward;

	#[tokio::test]
	async fn forward_forwards_errors() {
		let mut job = Job::builder_simple::<(), _>("test")
			.action(transform_fn(async |_| {
				Err::<Entry, _>(
					Box::new(io::Error::other("other error")) as Box<dyn Error + Send + Sync>
				)
			}))
			.error_handling(Forward)
			.trigger(())
			.cancel_token(None)
			.build();

		let JobResult::Err(error) = job.run().await else {
			panic!("Job didn't return an error");
		};

		let FetcherError::Transform(transform_error) = error.first() else {
			panic!("Unexpected error type");
		};

		assert_matches!(
			&**transform_error,
			TransformError {
				kind: TransformErrorKind::Other(_),
				..
			}
		);
	}
}
