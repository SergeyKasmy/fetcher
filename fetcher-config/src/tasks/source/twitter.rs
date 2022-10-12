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
pub struct Twitter(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<Inner>);

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Inner {
	pub handle: String,
	pub filter: Vec<String>,
}

impl Twitter {
	pub fn parse(self, external: &dyn ExternalData) -> Result<CWithSharedRF, ConfigError> {
		let twitter_sources = self
			.0
			.into_iter()
			.map(|x| Ok(CWithSharedRFKind::Twitter(x.parse(external)?)))
			.collect::<Result<_, ConfigError>>()?;

		Ok(CWithSharedRF::new(twitter_sources)
			.expect("should always be the same since we are deserializing only Twitter here"))
	}
}

impl Inner {
	pub fn parse(self, external: &dyn ExternalData) -> Result<CTwitter, ConfigError> {
		let (api_key, api_secret) = external
			.twitter_token()?
			.ok_or(ConfigError::TwitterApiKeysMissing)?;

		Ok(CTwitter::new(self.handle, api_key, api_secret, self.filter))
	}
}
