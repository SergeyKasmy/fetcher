/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{
	jobs::external_data::{ExternalDataResult, ProvideExternalData},
	FetcherConfigError as ConfigError,
};
use fetcher_core::source::Twitter as CTwitter;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

#[serde_as]
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Default, Debug)]
#[serde(transparent)]
pub struct Twitter(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<String>);

impl Twitter {
	pub fn parse<D>(self, external: &D) -> Result<Vec<CTwitter>, ConfigError>
	where
		D: ProvideExternalData + ?Sized,
	{
		let (api_key, api_secret) = match external.twitter_token() {
			ExternalDataResult::Ok(v) => v,
			ExternalDataResult::Unavailable => return Err(ConfigError::TwitterApiKeysMissing),
			ExternalDataResult::Err(e) => return Err(e.into()),
		};

		self.0
			.into_iter()
			.map(|handle| Ok(CTwitter::new(handle, api_key.clone(), api_secret.clone())))
			.collect()
	}
}
