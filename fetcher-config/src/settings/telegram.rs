/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Telegram {
	token: String,
}

impl Telegram {
	pub fn parse(self) -> String {
		let Self { token } = self;

		token
	}

	pub fn unparse(token: String) -> Self {
		Self { token }
	}
}
