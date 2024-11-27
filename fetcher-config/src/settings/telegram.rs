/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Telegram {
	pub token: String,
}

impl Telegram {
	#[must_use]
	pub fn decode_from_conf(self) -> String {
		let Self { token } = self;

		token
	}

	#[must_use]
	pub fn encode_into_conf(token: String) -> Self {
		Self { token }
	}
}
