/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::auth::GoogleAuth;
use crate::error::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct TwitterCfg {
	pub key: String,
	pub secret: String,
}
