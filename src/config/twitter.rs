/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;

use crate::{
	error::{Error, Result},
	settings, source,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Twitter {
	pretty_name: String,
	handle: String,
	filter: Vec<String>,
}

impl TryFrom<Twitter> for source::Twitter {
	type Error = Error;

	fn try_from(v: Twitter) -> Result<Self> {
		let (api_key, api_secret) = settings::twitter()?;

		source::Twitter::new(v.pretty_name, v.handle, api_key, api_secret, v.filter)
	}
}
