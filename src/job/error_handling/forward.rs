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

		HandleErrorResult::StopAndReturnErrs(errors)
	}
}
