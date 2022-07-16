/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::error::Error as StdError;
use std::fmt::Write as _;

pub mod sink;
pub mod source;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Can't fetch data")]
	Source(#[from] source::Error),

	#[error("Can't send data")]
	Sink(#[from] sink::Error),

	#[error("Google authentication error")]
	GoogleOAuth2(#[from] GoogleOAuth2Error),

	#[error("Error writing to the external read filter")]
	ReadFilterExternalWrite(#[source] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum GoogleOAuth2Error {
	#[error("Error contacting Google servers for authentication")]
	Post(#[source] reqwest::Error), // TODO: maybe integrate with source::HttpError?

	#[error("{0}")]
	Auth(String),
}

impl Error {
	/// Return some network connection error if it is some, otherwise return None
	#[allow(clippy::match_same_arms)]
	#[must_use]
	pub fn is_connection_error(&self) -> Option<&(dyn StdError + Send + Sync)> {
		match self {
			Error::Source(source_err) => match source_err {
				source::Error::Parse(_) => None,
				source::Error::EmptySourceList => None,
				source::Error::FileRead(_, _) => None,
				source::Error::Http(http_err) => Some(http_err),
				// I know it will match any future variants automatically but I actually want it to do that anyways
				#[allow(clippy::match_wildcard_for_single_variants)]
				source::Error::Email(email_err) => match &**email_err {
					source::EmailError::Imap(imap_err) => Some(imap_err),
					_ => None,
				},
				source::Error::Twitter(source::TwitterError::Other(other_twitter_err)) => {
					Some(other_twitter_err)
				}
				source::Error::Twitter(_) => None,
			},
			Error::Sink(sink_err) => match sink_err {
				sink::Error::StdoutWrite(_) => None,
				sink::Error::Telegram {
					source: teloxide::RequestError::Network(teloxide_network_err),
					..
				} => Some(teloxide_network_err),
				sink::Error::Telegram { .. } => None,
			},
			Error::GoogleOAuth2(google_oauth2_err) => match google_oauth2_err {
				GoogleOAuth2Error::Post(post_err) => Some(post_err),
				GoogleOAuth2Error::Auth(_) => None,
			},
			Error::ReadFilterExternalWrite(_) => None,
		}
	}
}

pub trait ErrorChainExt {
	fn display_chain(&self) -> String;
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
