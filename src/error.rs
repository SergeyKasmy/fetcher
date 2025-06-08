/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that [`fetcher`](`crate`) can emit

mod error_chain_display;
pub(crate) mod error_trait;

pub use self::{error_chain_display::ErrorChainDisplay, error_trait::Error};

use either::Either;

use crate::{
	actions::{filters::error::FilterError, transforms::error::TransformError},
	auth::AuthError,
	external_save::ExternalSaveError,
	sinks::error::SinkError,
	sources::error::SourceError,
};

use std::{convert::Infallible, error::Error as StdError};

// TODO: attach backtraces to all inner errors
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum FetcherError {
	#[error("Unable to fetch entries or mark an entry as read")]
	Source(#[from] SourceError),

	#[error("Unable to transform entries")]
	Transform(#[from] Box<TransformError>),

	#[error("Unable to filter some entries out")]
	Filter(#[from] FilterError),

	#[error("Unable to send a message to sink")]
	Sink(#[from] SinkError),

	#[error("Authentication failure")]
	Auth(#[from] AuthError),

	#[error(transparent)]
	ExternalSave(#[from] ExternalSaveError),

	#[error(transparent)]
	Other(#[from] Box<dyn Error>),
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("Invalid URL: {1}")]
pub struct InvalidUrlError(#[source] pub url::ParseError, pub String);

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("Invalid regular expression")]
pub struct BadRegexError(#[from] pub regex::Error);

impl Error for FetcherError {
	fn is_network_related(&self) -> Option<&dyn Error> {
		// I know it will match any future variants automatically but I actually want it to do that anyways
		#[expect(clippy::match_same_arms)]
		match self {
			Self::Source(e) => e.is_network_related(),
			Self::Transform(e) => e.is_network_related(),
			Self::Sink(e) => e.is_network_related(),
			Self::Auth(e) => e.is_network_related(),
			Self::Filter(e) => e.is_network_related(),
			Self::ExternalSave(_) => None,
			Self::Other(e) => e.is_network_related(),
		}
	}
}

impl From<TransformError> for FetcherError {
	fn from(e: TransformError) -> Self {
		FetcherError::Transform(Box::new(e))
	}
}

impl From<Box<dyn StdError + Send + Sync>> for FetcherError {
	fn from(value: Box<dyn StdError + Send + Sync>) -> Self {
		Self::Other(value.into())
	}
}

impl From<Infallible> for FetcherError {
	fn from(value: Infallible) -> Self {
		match value {}
	}
}

#[cfg(feature = "nightly")]
impl From<!> for FetcherError {
	fn from(value: !) -> Self {
		match value {}
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
