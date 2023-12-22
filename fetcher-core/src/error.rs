/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that [`fetcher`](`crate`) can emit

use crate::{
	action::transform::error::TransformError, auth::google::GoogleOAuth2Error,
	external_save::ExternalSaveError, sink::error::SinkError, source::error::SourceError,
};

use std::error::Error as StdError;

#[allow(missing_docs)] // error message is self-documenting
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

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
#[error("Invalid URL: {1}")]
pub struct InvalidUrlError(#[source] pub url::ParseError, pub String);

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
#[error("Invalid regular expression")]
pub struct BadRegexError(#[from] pub regex::Error);

impl FetcherError {
	/// Check if the current error is somehow related to network connection and return it if it is
	#[allow(clippy::match_same_arms)]
	#[must_use]
	pub fn is_connection_error(&self) -> Option<&(dyn StdError + Send + Sync)> {
		// I know it will match any future variants automatically but I actually want it to do that anyways
		#[allow(clippy::match_wildcard_for_single_variants)]
		match self {
			Self::Source(source_err) => source_err.is_connection_err(),
			FetcherError::Transform(tr_err) => tr_err.is_connection_err(),
			FetcherError::Sink(sink_err) => sink_err.is_connection_err(),
			FetcherError::GoogleOAuth2(google_oauth2_err) => google_oauth2_err.is_connection_err(),
			_ => None,
		}
	}
}

impl From<TransformError> for FetcherError {
	fn from(e: TransformError) -> Self {
		FetcherError::Transform(Box::new(e))
	}
}
