/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that [`fetcher`](`crate`) can emit

use either::Either;

use crate::actions::filters::error::FilterError;
use crate::{
	actions::transforms::error::TransformError, auth::google::GoogleOAuth2Error,
	external_save::ExternalSaveError, sinks::error::SinkError, sources::error::SourceError,
};

use std::fmt::Display;
use std::{convert::Infallible, error::Error as StdError};

// TODO: attach backtraces to all inner errors
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum FetcherError {
	#[error("Can't fetch data")]
	Source(#[from] SourceError),

	#[error("Can't transform entries")]
	Transform(#[from] Box<TransformError>),

	#[error("Can't filter entries")]
	Filter(#[from] FilterError),

	#[error("Can't send messages")]
	Sink(#[from] SinkError),

	#[error("Google authentication error")]
	GoogleOAuth2(#[from] GoogleOAuth2Error),

	#[error("Error writing to the external save location")]
	ExternalSave(#[source] ExternalSaveError),

	#[error("Other error")]
	Other(#[from] Box<dyn StdError + Send + Sync>),
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("Invalid URL: {1}")]
pub struct InvalidUrlError(#[source] pub url::ParseError, pub String);

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("Invalid regular expression")]
pub struct BadRegexError(#[from] pub regex::Error);

impl FetcherError {
	/// Checks if the current error is somehow related to network connection and return it if it is
	#[must_use]
	pub fn is_connection_error(&self) -> Option<&(dyn StdError + Send + Sync)> {
		// I know it will match any future variants automatically but I actually want it to do that anyways
		match self {
			Self::Source(e) => e.is_connection_err(),
			Self::Transform(e) => e.is_connection_err(),
			Self::Sink(e) => e.is_connection_err(),
			Self::GoogleOAuth2(e) => e.is_connection_err(),
			_ => None,
		}
	}
}

impl From<TransformError> for FetcherError {
	fn from(e: TransformError) -> Self {
		FetcherError::Transform(Box::new(e))
	}
}

impl From<Infallible> for FetcherError {
	fn from(_: Infallible) -> Self {
		unreachable!()
	}
}

impl<A, B> From<Either<A, B>> for FetcherError
where
	A: Into<FetcherError>,
	B: Into<FetcherError>,
{
	fn from(value: Either<A, B>) -> Self {
		match value {
			Either::Left(a) => a.into(),
			Either::Right(b) => b.into(),
		}
	}
}

/// Wrapper around a type implementing [`std::error::Error`]
/// that provides a pretty [`Display`] implementation.
///
/// It may looked like this:
///
/// Error 1
///
/// Caused by:
///   1: Error 2
///   2: Error 3
///   3: Error 4
pub struct ErrorChainDisplay<'a>(pub &'a dyn StdError);

impl Display for ErrorChainDisplay<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut current_err = self.0;
		let mut counter = 0;
		write!(f, "{current_err}")?;

		while let Some(source) = StdError::source(current_err) {
			current_err = source;
			counter += 1;
			if counter == 1 {
				write!(f, "\n\nCaused by:")?;
			}

			write!(f, "\n\t{counter}: {current_err}")?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::FetcherError;

	fn takes_send<T: Send>() {}
	fn takes_sync<T: Sync>() {}

	#[test]
	fn assert_error_is_send_sync() {
		takes_send::<FetcherError>();
		takes_sync::<FetcherError>();
	}
}
