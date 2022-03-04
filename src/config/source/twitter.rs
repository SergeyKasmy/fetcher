/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;

use crate::{error::Result, settings, source};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Twitter {
	pretty_name: String,
	handle: String,
	filter: Vec<String>,
}

impl Twitter {
	pub(crate) fn parse(self) -> Result<source::Twitter> {
		let (api_key, api_secret) = settings::twitter()?;

		source::Twitter::new(
			self.pretty_name,
			self.handle,
			api_key,
			api_secret,
			self.filter,
		)
	}
}
