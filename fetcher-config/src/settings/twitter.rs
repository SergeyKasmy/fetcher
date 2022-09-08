/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Twitter {
	api_key: String,
	api_secret: String,
}

impl Twitter {
	#[must_use]
	pub fn parse(self) -> (String, String) {
		let Self {
			api_key,
			api_secret,
		} = self;

		(api_key, api_secret)
	}

	pub fn unparse(api_key: String, api_secret: String) -> Self {
		Self {
			api_key,
			api_secret,
		}
	}
}
