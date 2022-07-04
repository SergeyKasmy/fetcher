/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::{
	config::DataSettings,
	error::{Error, Result},
	source,
};

#[derive(Deserialize, Serialize, Debug)]
// #[serde(deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
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
			.ok_or_else(|| Error::ServiceNotReady("Twitter authentication".to_owned()))?;

		Ok(source::Twitter::new(
			self.pretty_name,
			self.handle,
			api_key,
			api_secret,
			self.filter,
		))
	}
}
