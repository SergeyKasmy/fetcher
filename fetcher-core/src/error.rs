/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that [`fetcher`](`crate`) can emit

use crate::{
	action::transform::error::{self as transform_err, TransformError, TransformErrorKind},
	sink::error::SinkError,
	source::error::{EmailError, ImapError, RedditError, SourceError, TwitterError},
};

use roux::util::RouxError;
use std::{error::Error as StdError, fmt::Write as _};

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
	ReadFilterExternalWrite(#[source] std::io::Error),
}

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
#[error("Invalid URL: {1}")]
pub struct InvalidUrlError(#[source] pub url::ParseError, pub String);

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum GoogleOAuth2Error {
	#[error("Error contacting Google servers for authentication")]
	Post(#[source] reqwest::Error),

	/// An error received from Google, whatever it is
	#[error("{0}")]
	Auth(String),
}

// Re-exported in error::source and error::sink modules. Private in this one to avoid namespace pollution
pub(crate) mod exec_error {
	use std::{io, string::FromUtf8Error};

	/// Errors that happened while executing a process
	#[allow(missing_docs)] // error message is self-documenting
	#[derive(thiserror::Error, Debug)]
	pub enum ExecError {
		#[error("Bad command")]
		BadCommand(#[source] io::Error),

		#[error("Command output is not valid UTF-8")]
		BadUtf8(#[from] FromUtf8Error),

		#[error("Can't start the process")]
		CantStart(#[source] io::Error),

		#[error("Can't pass data to the stdin of the process")]
		CantWriteStdin(#[source] io::Error),
	}
}

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
			Error::Source(source_err) => match source_err {
				SourceError::EmptySourceList => None,
				SourceError::SourceListHasDifferentVariants => None,
				SourceError::FileRead(_, _) => None,
				SourceError::Http(_) => Some(self),
				SourceError::Email(email_err) => match &**email_err {
					EmailError::Imap(ImapError::TlsInitFailed(_)) => Some(self),
					EmailError::Imap(_) => None,
					_ => None,
				},
				SourceError::Twitter(twitter_err) => match twitter_err {
					TwitterError::Auth(egg_mode::error::Error::NetError(_)) => Some(self),
					TwitterError::Other(egg_mode::error::Error::NetError(_)) => Some(self),
					_ => None,
				},
				SourceError::Reddit(reddit_err) => match reddit_err {
					RedditError::Reddit(RouxError::Network(_)) => Some(self),
					_ => None,
				},
				SourceError::Exec(_) => None,
			},
			Error::Transform(tr_err) => match &tr_err.kind {
				TransformErrorKind::Http(transform_err::HttpError::Other(_)) => Some(self),
				_ => None,
			},
			Error::Sink(sink_err) => match sink_err {
				SinkError::Telegram {
					source: teloxide::RequestError::Network(_),
					..
				} => Some(self),
				SinkError::Telegram { .. } => None,
				SinkError::Exec(_) => None,
				SinkError::Stdout(_) => None,
			},
			Error::GoogleOAuth2(google_oauth2_err) => match google_oauth2_err {
				GoogleOAuth2Error::Post(_) => Some(self),
				GoogleOAuth2Error::Auth(_) => None,
			},
			Error::ReadFilterExternalWrite(_) => None,
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
