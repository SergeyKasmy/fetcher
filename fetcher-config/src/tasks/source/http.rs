/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::{error::source::HttpError as CHttpError, source::Http as CHttp};

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged, deny_unknown_fields)]
pub enum Http {
	Get(Url),
	Post { url: Url, post_body: String },
}

impl Http {
	pub fn parse(self) -> Result<CHttp, CHttpError> {
		match self {
			Http::Get(url) => CHttp::new_get(url),
			Http::Post { url, post_body } => CHttp::new_post(url, &post_body),
		}
	}
}
