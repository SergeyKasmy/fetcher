/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{tasks::external_data::ExternalData, Error as ConfigError};
use fetcher_core::source::{
	Twitter as CTwitter, WithSharedRF as CWithSharedRF, WithSharedRFKind as CWithSharedRFKind,
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Twitter(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<String>);

impl Twitter {
	pub fn parse(self, external: &dyn ExternalData) -> Result<CWithSharedRF, ConfigError> {
		let (api_key, api_secret) = external
			.twitter_token()?
			.ok_or(ConfigError::TwitterApiKeysMissing)?;

		let twitter_sources = self
			.0
			.into_iter()
			.map(|handle| {
				Ok(CWithSharedRFKind::Twitter(CTwitter::new(
					handle,
					api_key.clone(),
					api_secret.clone(),
				)))
			})
			.collect::<Result<_, ConfigError>>()?;

		Ok(CWithSharedRF::new(twitter_sources)
			.expect("should always be the same since we are deserializing only Twitter here"))
	}
}
