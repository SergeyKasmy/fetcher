/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use crate::tasks::TaskSettings;
use crate::Error;
use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
pub struct Twitter {
	handle: String,
	filter: Vec<String>,
}

impl Twitter {
	pub fn parse(self, settings: &dyn TaskSettings) -> Result<source::Twitter, Error> {
		let (api_key, api_secret) = settings
			.twitter_token()?
			.ok_or(Error::TwitterApiKeysMissing)?;

		Ok(source::Twitter::new(
			self.handle,
			api_key,
			api_secret,
			self.filter,
		))
	}
}
