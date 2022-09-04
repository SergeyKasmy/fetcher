/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use url::Url;

use fetcher_core::error;
use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Http {
	pub url: Url,
}

impl Http {
	pub fn parse(self) -> Result<source::Http, error::source::HttpError> {
		source::Http::new(self.url)
	}
}
