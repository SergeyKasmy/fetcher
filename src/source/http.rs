/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use once_cell::sync::Lazy;
use std::fmt::Debug;
use url::Url;

use crate::entry::Entry;
use crate::error::Result;
use crate::sink::Message;

static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
	reqwest::ClientBuilder::new()
		.timeout(std::time::Duration::from_secs(30))
		.build()
		.expect("TLS init error") // TODO: fail gracefully
});

pub struct Http {
	pub(crate) url: Url,
	client: reqwest::Client,
}

impl Http {
	#[must_use]
	pub fn new(url: Url) -> Self {
		Self {
			url,
			client: CLIENT.clone(),
		}
	}

	#[tracing::instrument(skip_all)]
	pub async fn get(&self) -> Result<Vec<Entry>> {
		tracing::debug!("Fetching HTTP source");

		tracing::trace!("Making a request to {:?}", self.url.as_str());
		let request = self.client.get(self.url.as_str()).send().await?;

		tracing::trace!("Getting text body of the responce");
		let page = request.text().await?;
		tracing::trace!("Done");

		Ok(vec![Entry {
			id: String::new(), // FIXME: use a proper id
			msg: Message {
				title: None,
				body: page,
				link: Some(self.url.clone()),
				media: None,
			},
		}])
	}
}

impl Debug for Http {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Http")
			.field("url", &self.url.as_str())
			.finish_non_exhaustive()
	}
}
