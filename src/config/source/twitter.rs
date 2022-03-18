/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::{config::DataSettings, error::Result, source};

#[derive(Deserialize, Serialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Twitter {
	pretty_name: String,
	handle: String,
	filter: Vec<String>,
}

impl Twitter {
	pub(crate) fn parse(self, settings: &DataSettings) -> Result<source::Twitter> {
		// let (api_key, api_secret) = settings::twitter()?;
		let (api_key, api_secret) = settings
			.twitter_auth
			.as_ref()
			.cloned()
			.expect("No twitter auth data"); // FIXME

		Ok(source::Twitter::new(
			self.pretty_name,
			self.handle,
			api_key,
			api_secret,
			self.filter,
		))
	}
}
