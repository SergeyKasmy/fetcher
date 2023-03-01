/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that [`fetcher`](`crate`) can emit

use crate::{
	action::transform::error::TransformError, auth::google::GoogleOAuth2Error,
	sink::error::SinkError, source::error::SourceError,
};

use std::{error::Error as StdError, fmt::Write, io};

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Can't fetch data")]
	Source(#[from] SourceError),

	#[error("Can't transform data")]
	Transform(#[from] Box<TransformError>),

	#[error("Can't send data")]
	Sink(#[from] SinkError),

	#[error("Google authentication error")]
	GoogleOAuth2(#[from] GoogleOAuth2Error),

	#[error("Error writing to the external read filter")]
	ReadFilterExternalWrite(#[source] io::Error),
}

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
#[error("Invalid URL: {1}")]
pub struct InvalidUrlError(#[source] pub url::ParseError, pub String);

/// Extention trait for [`std::error::Error`] to print the entire chain of the error
pub trait ErrorChainExt {
	/// Return a string intented for logging or printing that formats an error's entire error source chain
	fn display_chain(&self) -> String;
}

impl Error {
	/// Check if the current error is somehow related to network connection and return it if it is
	#[allow(clippy::match_same_arms)]
	#[must_use]
	pub fn is_connection_error(&self) -> Option<&(dyn StdError + Send + Sync)> {
		// I know it will match any future variants automatically but I actually want it to do that anyways
		#[allow(clippy::match_wildcard_for_single_variants)]
		match self {
			Self::Source(source_err) => source_err.is_connection_err(),
			Error::Transform(tr_err) => tr_err.is_connection_err(),
			Error::Sink(sink_err) => sink_err.is_connection_err(),
			Error::GoogleOAuth2(google_oauth2_err) => google_oauth2_err.is_connection_err(),
			_ => None,
		}
	}
}

impl<T: StdError> ErrorChainExt for T {
	#[must_use]
	fn display_chain(&self) -> String {
		let mut current_err: &dyn StdError = self;
		let mut counter = 0;
		let mut output = format!("{current_err}");

		while let Some(source) = StdError::source(current_err) {
			current_err = source;
			counter += 1;
			if counter == 1 {
				let _ = write!(output, "\n\nCaused by:");
			}

			let _ = write!(output, "\n\t{counter}: {current_err}");
		}

		output
	}
}

impl From<TransformError> for Error {
	fn from(e: TransformError) -> Self {
		Error::Transform(Box::new(e))
	}
}
