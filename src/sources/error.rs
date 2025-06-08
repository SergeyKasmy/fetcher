/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that can happen in the (`parent`)[`super`] module

pub use crate::exec::ExecError;

use crate::{
	error::{Error, error_trait::BoxErrorWrapper},
	read_filter::mark_as_read::MarkAsReadError,
};

#[cfg(feature = "source-http")]
use super::http::HttpError;

#[cfg(feature = "source-email")]
use super::email::{EmailError, ImapError};

#[cfg(feature = "source-reddit")]
use {super::reddit::RedditError, roux::util::RouxError};

use std::{convert::Infallible, error::Error as StdError, path::PathBuf};

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum SourceError {
	#[error("Failed to mark an entry as read")]
	MarkAsRead(#[from] MarkAsReadError),

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

	#[error(transparent)]
	Other(#[from] Box<dyn Error>),
}

impl Error for SourceError {
	fn is_network_related(&self) -> Option<&dyn Error> {
		#[allow(clippy::match_same_arms, reason = "clearer code")]
		match self {
			Self::MarkAsRead(e) if e.is_network_related().is_some() => Some(self),
			#[cfg(feature = "source-http")]
			Self::Http(_) => Some(self),
			#[cfg(feature = "source-email")]
			Self::Email(email_err) => match &**email_err {
				EmailError::Imap(ImapError::ConnectionFailed(_)) => Some(self),
				_ => None,
			},
			#[cfg(feature = "source-reddit")]
			Self::Reddit(RedditError::Reddit(RouxError::Network(_))) => Some(self),
			Self::Other(other_err) if other_err.is_network_related().is_some() => Some(self),
			_ => None,
		}
	}
}

#[cfg(feature = "source-email")]
impl From<EmailError> for SourceError {
	fn from(e: EmailError) -> Self {
		Self::Email(Box::new(e))
	}
}

impl From<Box<dyn StdError + Send + Sync>> for SourceError {
	fn from(value: Box<dyn StdError + Send + Sync>) -> Self {
		Self::Other(Box::new(BoxErrorWrapper(value)))
	}
}

impl From<Infallible> for SourceError {
	fn from(value: Infallible) -> Self {
		match value {}
	}
}

#[cfg(feature = "nightly")]
impl From<!> for SourceError {
	fn from(value: !) -> Self {
		match value {}
	}
}
