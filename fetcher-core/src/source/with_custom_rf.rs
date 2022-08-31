/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod email;

use self::email::Email;
use crate::{
	entry::Entry,
	error::source::{EmailError, Error as SourceError},
};

#[derive(Debug)]
pub enum Source {
	Email(Email),
}

impl Source {
	/// Fetch all entries from the source
	///
	/// # Errors
	/// if there was an error fetching from the source (such as a network connection error or maybe even an authentication error)
	pub async fn get(&mut self) -> Result<Vec<Entry>, SourceError> {
		Ok(match self {
			Self::Email(x) => x.get().await.map_err(Box::new)?,
		})
	}

	/// Delegate for [`Source::mark_as_read`]
	#[allow(clippy::missing_errors_doc)]
	pub async fn mark_as_read(&mut self, id: &str) -> Result<(), SourceError> {
		match self {
			Self::Email(x) => x
				.mark_as_read(id)
				.await
				.map_err(|e| Box::new(EmailError::Imap(e)))?,
		};

		Ok(())
	}

	/// Delegate for [`Source::remove_read`]
	#[allow(clippy::ptr_arg)]
	pub fn remove_read(&self, _entries: &mut Vec<Entry>) {
		match self {
			Self::Email(_) => (), // NO-OP, emails should already be unread only when fetching
		}
	}
}
