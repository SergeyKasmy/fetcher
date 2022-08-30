/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// TODO: add google calendar source. Google OAuth2 is already implemented :)

pub mod with_custom_rf;
pub mod with_shared_rf;

pub use self::with_custom_rf::email::Email;
pub use self::with_shared_rf::file::File;
pub use self::with_shared_rf::http::Http;
pub use self::with_shared_rf::twitter::Twitter;

use crate::entry::Entry;
use crate::error::source::Error as SourceError;

#[derive(Debug)]
pub enum Source {
	WithSharedReadFilter(with_shared_rf::Source),
	WithCustomReadFilter(with_custom_rf::Source),
}

impl Source {
	/// Get all available entries from the source and run them through the parsers
	///
	/// # Errors
	/// * if there was an error fetching from the source
	/// * if there was an error parsing the just fetched entries
	pub async fn get(
		&mut self,
		// transforms: Option<&[Transform]>,
	) -> Result<Vec<Entry>, SourceError> {
		match self {
			Source::WithSharedReadFilter(x) => x.get().await,
			Source::WithCustomReadFilter(x) => x.get().await,
		}
	}
}
