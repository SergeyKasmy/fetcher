/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that can happen in the (`parent`)[`super`] module

pub use crate::exec::ExecError;

#[cfg(feature = "source-http")]
use super::http::HttpError;

#[cfg(feature = "source-email")]
use super::email::{EmailError, ImapError};

#[cfg(feature = "source-reddit")]
use {super::reddit::RedditError, roux::util::RouxError};

use std::{error::Error as StdError, path::PathBuf};

// TODO: Add "Other" error (Box<dyn Error>) for use for external source impls
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum SourceError {
	#[error("Can't read file {}", .1.to_string_lossy())]
	File(#[source] std::io::Error, PathBuf),

	#[error("Exec error")]
	Exec(#[from] ExecError),

	#[cfg(feature = "source-http")]
	#[error("HTTP error")]
	Http(#[from] HttpError),

	#[cfg(feature = "source-email")]
	#[error("Email error")]
	Email(#[from] Box<EmailError>),

	#[cfg(feature = "source-reddit")]
	#[error("Reddit error")]
	Reddit(#[from] RedditError),

	#[error("Other error")]
	Other(#[from] Box<dyn StdError + Send + Sync>),
}

#[cfg(feature = "source-email")]
impl From<EmailError> for SourceError {
	fn from(e: EmailError) -> Self {
		SourceError::Email(Box::new(e))
	}
}

impl SourceError {
	pub(crate) fn is_connection_err(&self) -> Option<&(dyn StdError + Send + Sync)> {
		#[expect(clippy::match_same_arms, reason = "clearer code")]
		match self {
			#[cfg(feature = "source-http")]
			Self::Http(_) => Some(self),
			#[cfg(feature = "source-email")]
			Self::Email(email_err) => match &**email_err {
				EmailError::Imap(ImapError::ConnectionFailed(_)) => Some(self),
				_ => None,
			},
			#[cfg(feature = "source-reddit")]
			Self::Reddit(RedditError::Reddit(RouxError::Network(_))) => Some(self),
			_ => None,
		}
	}
}
