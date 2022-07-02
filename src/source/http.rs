/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::fmt::Debug;
use url::Url;

use crate::entry::Entry;
use crate::error::Result;
use crate::sink::message::{Link, LinkLocation};
use crate::sink::Message;

pub struct Http {
	pub(crate) url: Url,
}

impl Http {
	#[tracing::instrument(skip_all)]
	pub async fn get(&self) -> Result<Vec<Entry>> {
		tracing::debug!("Fetching HTML source");

		let page = reqwest::get(self.url.as_str()).await?.text().await?;

		Ok(vec![Entry {
			id: String::new(), // FIXME: use a proper id
			msg: Message {
				title: None,
				body: page,
				link: Some(Link {
					url: self.url.clone(),
					loc: LinkLocation::Bottom,
				}),
				media: None,
			},
		}])
	}
}

impl Debug for Http {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Http")
			.field("url", &self.url.as_str())
			.finish()
	}
}
