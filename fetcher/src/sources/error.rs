/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that can happen in the (`parent`)[`super`] module

pub use crate::exec::ExecError;

use super::{
	email::{EmailError, ImapError},
	http::HttpError,
	reddit::RedditError,
};

use roux::util::RouxError;
use std::{error::Error as StdError, path::PathBuf};

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum SourceError {
	#[error("Can't read file {}", .1.to_string_lossy())]
	File(#[source] std::io::Error, PathBuf),

	#[error("HTTP error")]
	Http(#[from] HttpError),

	#[error("Email error")]
	Email(#[from] Box<EmailError>),

	#[error("Reddit error")]
	Reddit(#[from] RedditError),

	#[error("Exec error")]
	Exec(#[from] ExecError),

	#[error("This is a debug error automatically triggered for debugging purposes")]
	Debug,
}

impl From<EmailError> for SourceError {
	fn from(e: EmailError) -> Self {
		SourceError::Email(Box::new(e))
	}
}

impl SourceError {
	pub(crate) fn is_connection_err(&self) -> Option<&(dyn StdError + Send + Sync)> {
		#[expect(clippy::match_same_arms, reason = "clearer code")]
		match self {
			Self::Http(_) => Some(self),
			Self::Email(email_err) => match &**email_err {
				EmailError::Imap(ImapError::ConnectionFailed(_)) => Some(self),
				_ => None,
			},
			Self::Reddit(RedditError::Reddit(RouxError::Network(_))) => Some(self),
			_ => None,
		}
	}
}
