/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`LogAndIgnore`] error handler

use std::convert::Infallible;

use crate::error::FetcherError;
use crate::job::ErrorChainDisplay;

use super::{HandleError, HandleErrorContext, HandleErrorResult};

/// Error handler that logs all errors and continues job execution.
pub struct LogAndIgnore;

impl HandleError for LogAndIgnore {
	type HandlerErr = Infallible;

	async fn handle_errors(
		&mut self,
		errors: Vec<FetcherError>,
		_cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::HandlerErr> {
		for error in &errors {
			tracing::error!("{}", ErrorChainDisplay(error));
		}

		HandleErrorResult::ContinueJob
	}
}
