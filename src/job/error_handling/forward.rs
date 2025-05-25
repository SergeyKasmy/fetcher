/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Forward`] error handler

use std::convert::Infallible;

use super::{HandleError, HandleErrorContext, HandleErrorResult};
use crate::error::FetcherError;

/// Error handler that forwards all errors to the caller, stopping the job immediately.
pub struct Forward;

impl HandleError for Forward {
	type HandlerErr = Infallible;

	async fn handle_errors(
		&mut self,
		errors: Vec<FetcherError>,
		_cx: HandleErrorContext<'_>,
	) -> HandleErrorResult<Self::HandlerErr> {
		tracing::trace!("Forwarding errors");

		HandleErrorResult::StopAndReturnErrs(errors)
	}
}
