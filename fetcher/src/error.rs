/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that [`fetcher`](`crate`) can emit

use crate::{
	action::transforms::error::TransformError, auth::google::GoogleOAuth2Error,
	external_save::ExternalSaveError, sink::error::SinkError, source::error::SourceError,
};

use std::{convert::Infallible, error::Error as StdError};

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum FetcherError {
	#[error("Can't fetch data")]
	Source(#[from] SourceError),

	#[error("Can't transform data")]
	Transform(#[from] Box<TransformError>),

	#[error("Can't send data")]
	Sink(#[from] SinkError),

	#[error("Google authentication error")]
	GoogleOAuth2(#[from] GoogleOAuth2Error),

	#[error("Error writing to the external save location")]
	ExternalSave(#[source] ExternalSaveError),
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
		#[expect(
			clippy::match_wildcard_for_single_variants,
			reason = "all other branches are ignored, no matter how many there are"
		)]
		match self {
			Self::Source(e) => e.is_connection_err(),
			FetcherError::Transform(e) => e.is_connection_err(),
			FetcherError::Sink(e) => e.is_connection_err(),
			FetcherError::GoogleOAuth2(e) => e.is_connection_err(),
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
