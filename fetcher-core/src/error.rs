/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that [`fetcher`](`crate`) can emit

pub mod sink;
pub mod source;
pub mod transform;

use roux::util::RouxError;

use self::{
	source::{ImapError, RedditError, TwitterError},
	transform::Error as TransformError,
};

use std::{error::Error as StdError, fmt::Write as _};

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Can't fetch data")]
	Source(#[from] source::Error),

	#[error("Can't transform data")]
	Transform(#[from] Box<TransformError>),

	#[error("Can't send data")]
	Sink(#[from] sink::Error),

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
				source::Error::EmptySourceList => None,
				source::Error::SourceListHasDifferentVariants => None,
				source::Error::FileRead(_, _) => None,
				source::Error::Http(_) => Some(self),
				source::Error::Email(email_err) => match &**email_err {
					source::EmailError::Imap(ImapError::TlsInitFailed(_)) => Some(self),
					source::EmailError::Imap(_) => None,
					_ => None,
				},
				source::Error::Twitter(twitter_err) => match twitter_err {
					TwitterError::Auth(egg_mode::error::Error::NetError(_)) => Some(self),
					TwitterError::Other(egg_mode::error::Error::NetError(_)) => Some(self),
					_ => None,
				},
				source::Error::Reddit(reddit_err) => match reddit_err {
					RedditError::Reddit(RouxError::Network(_)) => Some(self),
					_ => None,
				},
				source::Error::Exec(_) => None,
			},
			Error::Transform(tr_err) => match &tr_err.kind {
				transform::Kind::Http(transform::HttpError::Other(_)) => Some(self),
				_ => None,
			},
			Error::Sink(sink_err) => match sink_err {
				sink::Error::Telegram {
					source: teloxide::RequestError::Network(_),
					..
				} => Some(self),
				sink::Error::Telegram { .. } => None,
				sink::Error::Exec(_) => None,
				sink::Error::Stdout(_) => None,
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
