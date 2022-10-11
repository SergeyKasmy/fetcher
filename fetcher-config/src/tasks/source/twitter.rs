/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::tasks::external_data::ExternalData;
use crate::Error;
use fetcher_core::source;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Twitter {
	handle: String,
	filter: Vec<String>,
}

impl Twitter {
	pub fn parse(self, external: &dyn ExternalData) -> Result<source::Twitter, Error> {
		let (api_key, api_secret) = external
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
