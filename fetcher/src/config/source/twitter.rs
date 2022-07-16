/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use crate::config::DataSettings;
use crate::error::ConfigError;
use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
// #[serde(deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
pub(crate) struct Twitter {
	handle: String,
	filter: Vec<String>,
}

impl Twitter {
	pub(crate) fn parse(self, settings: &DataSettings) -> Result<source::Twitter, ConfigError> {
		let (api_key, api_secret) = settings
			.twitter_auth
			.as_ref()
			.cloned()
			.ok_or(ConfigError::TwitterApiKeysMissing)?;

		Ok(source::Twitter::new(
			self.handle,
			api_key,
			api_secret,
			self.filter,
		))
	}
}
