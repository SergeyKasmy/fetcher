/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`LogAndIgnore`] error handler

use std::convert::Infallible;

use non_non_full::NonEmptyVec;

use crate::job::ErrorChainDisplay;
use crate::{error::FetcherError, maybe_send::MaybeSync};

use super::{HandleError, HandleErrorContext, HandleErrorResult};

/// Error handler that logs all errors and continues job execution.
#[derive(Clone, Copy, Debug)]
pub struct LogAndIgnore;

impl<Tr: MaybeSync> HandleError<Tr> for LogAndIgnore {
	type HandlerErr = Infallible;

	async fn handle_errors(
		&mut self,
		errors: NonEmptyVec<FetcherError>,
		_cx: HandleErrorContext<'_, Tr>,
	) -> HandleErrorResult<Self::HandlerErr> {
		for error in &errors {
			tracing::error!("{}", ErrorChainDisplay(error));
		}

		HandleErrorResult::ContinueJob
	}
}

#[cfg(test)]
mod tests {
	use assert_matches::assert_matches;

	use std::{error::Error, io};

	use crate::{
		Job,
		actions::transform_fn,
		entry::Entry,
		job::{JobResult, error_handling::LogAndIgnore},
	};

	#[tokio::test]
	async fn log_and_ignore_ignores_error() {
		let mut job = Job::builder_simple::<(), _>("test")
			.action(transform_fn(async |_| {
				Err::<Entry, _>(
					Box::new(io::Error::other("other error")) as Box<dyn Error + Send + Sync>
				)
			}))
			.error_handling(LogAndIgnore)
			.trigger(())
			.cancel_token(None)
			.build();

		let result = job.run().await;
		assert_matches!(result, JobResult::Ok);
	}
}
