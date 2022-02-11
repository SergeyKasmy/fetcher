/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;
use url::Url;

use crate::source;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rss {
	url: Url,
}

impl From<Rss> for source::Rss {
	fn from(v: Rss) -> Self {
		source::Rss::new(v.url.to_string())
	}
}
