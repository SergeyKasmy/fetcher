/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};
use url::Url;

use crate::source;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Http {
	pub(crate) url: Url,
}

impl Http {
	pub(crate) fn parse(self) -> source::Http {
		source::Http::new(self.url)
	}
}
