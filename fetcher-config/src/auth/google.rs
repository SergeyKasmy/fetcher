/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use fetcher_core::auth;

#[derive(Serialize, Deserialize, Debug)]
pub struct Google {
	pub client_id: String,
	pub client_secret: String,
	pub refresh_token: String,
}

impl Google {
	#[must_use]
	pub fn parse(self) -> auth::Google {
		auth::Google::new(self.client_id, self.client_secret, self.refresh_token)
	}
}
