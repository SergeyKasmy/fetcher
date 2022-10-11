/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmailPassword {
	password: String,
}

impl EmailPassword {
	pub fn parse(self) -> String {
		let Self { password } = self;

		password
	}

	pub fn unparse(password: String) -> Self {
		Self { password }
	}
}
