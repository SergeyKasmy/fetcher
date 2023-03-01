/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::error::{exec_error::ExecError, InvalidUrlError};

use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum SourceError {
	#[error("Can't create a source with an empty source list")]
	EmptySourceList,

	#[error("Can't create a source with a source list that contains different source variants")]
	SourceListHasDifferentVariants,

	#[error("Can't read file {}", .1.to_string_lossy())]
	FileRead(#[source] std::io::Error, PathBuf),

	#[error("HTTP error")]
	Http(#[from] HttpError),

	#[error("Email error")]
	Email(#[from] Box<EmailError>),

	#[error("Twitter error")]
	Twitter(#[from] TwitterError),

	#[error("Reddit error")]
	Reddit(#[from] RedditError),

	#[error("Exec error")]
	Exec(#[from] ExecError),
}

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
	#[error("Invalid JSON for the POST request")]
	BadJson(#[from] serde_json::Error),

	#[error("Failed to init TLS")]
	TlsInitFailed(#[source] reqwest::Error),

	#[error("Can't send an HTTP request to {1:?}")]
	BadRequest(#[source] reqwest::Error, String),
}

#[allow(clippy::large_enum_variant)] // the entire enum is already boxed up above
#[derive(thiserror::Error, Debug)]
pub enum EmailError {
	#[error("IMAP connection error")]
	Imap(#[from] ImapError),

	#[error("Error parsing email")]
	Parse(#[from] mailparse::MailParseError),
}

#[derive(thiserror::Error, Debug)]
pub enum ImapError {
	#[error("Failed to init TLS")]
	TlsInitFailed(#[source] imap::Error),

	#[error(transparent)]
	GoogleAuth(Box<crate::error::Error>),

	#[error("Authentication error")]
	Auth(#[source] imap::Error),

	#[error(transparent)]
	Other(#[from] imap::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum TwitterError {
	#[error("Authentication failed")]
	Auth(#[source] egg_mode::error::Error),

	#[error(transparent)]
	Other(#[from] egg_mode::error::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum RedditError {
	#[error(transparent)]
	Reddit(#[from] roux::util::RouxError),

	#[error("Reddit API returned an invalid URL to a post/post's contents, which really shouldn't happen...")]
	InvalidUrl(#[from] InvalidUrlError),
}

impl From<EmailError> for SourceError {
	fn from(e: EmailError) -> Self {
		SourceError::Email(Box::new(e))
	}
}
