/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use once_cell::sync::Lazy;
use std::fmt::Debug;
use std::sync::RwLock;
use url::Url;

use crate::entry::Entry;
use crate::error::source::HttpError;
use crate::error::transform::HttpError as HttpTransformError;
use crate::sink::Message;

const USER_AGENT: &str =
	"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:96.0) Gecko/20100101 Firefox/96.0";

// option because tls init could've failed and we took out the Err()
static CLIENT: Lazy<RwLock<Option<Result<reqwest::Client, HttpError>>>> = Lazy::new(|| {
	RwLock::new(Some(
		reqwest::ClientBuilder::new()
			.timeout(std::time::Duration::from_secs(30))
			.build()
			.map_err(HttpError::TlsInitFailed),
	))
});

pub struct Http {
	pub(crate) url: Url,
	client: reqwest::Client,
}

impl Http {
	/// Create a new Http client
	///
	/// # Errors
	/// if TLS couldn't be initialized
	///
	/// # Panics
	/// This function may panic if a different thread crashes when calling this function
	pub fn new(url: Url) -> Result<Self, HttpError> {
		// take out the error out of the option if there was an error, otherwise just clone the Client
		let client = if let Ok(client) = CLIENT // if there was no error building the client
			.read()
			.expect("RwLock has been poisoned")
			.as_ref()
			// there was an error building the client and the error has already been extracted
			.ok_or(HttpError::ClientNotInitialized)?
		{
			client.clone()
		} else {
			return Err(CLIENT
				.write()
				.expect("RwLock has been poisoned")
				.take()
				.unwrap()
				.unwrap_err()); // should always be Some and Err because we .ok_or?'ed it up above
			    // TODO: remove these unwraps
		};

		Ok(Self { url, client })
	}

	// TODO: return a single entry, not a vec
	// TODO: set title from page title
	#[tracing::instrument(skip_all)]
	pub async fn get(&self) -> Result<Vec<Entry>, HttpError> {
		tracing::debug!("Fetching HTTP source");

		tracing::trace!("Making a request to {:?}", self.url.as_str());
		let request = self
			.client
			.get(self.url.as_str())
			.header(reqwest::header::USER_AGENT, USER_AGENT)
			.send()
			.await
			.map_err(|e| HttpError::Get(e, self.url.to_string()))?;

		tracing::trace!("Getting text body of the responce");
		let page = request
			.text()
			.await
			.map_err(|e| HttpError::Get(e, self.url.to_string()))?;
		tracing::trace!("Done. Body: ----------------------------------------\n{page:?}\n----------------------------------------\n");

		Ok(vec![Entry {
			id: None,
			msg: Message {
				title: None,
				body: page,
				link: Some(self.url.clone()),
				media: None,
			},
		}])
	}

	pub async fn transform(entry: &Entry) -> Result<Vec<Entry>, HttpTransformError> {
		let body = match Self::new(
			entry
				.msg
				.link
				.clone()
				.ok_or(HttpTransformError::MissingUrl)?,
		)?
		.get()
		.await?
		.pop()
		{
			Some(ent) => ent.msg.body,
			None => return Ok(Vec::new()), // TODO: is returning an empty vec really the move here
		};

		let entry = Entry {
			msg: Message {
				body,
				..Default::default()
			},
			..Default::default()
		};

		Ok(vec![entry])
	}
}

impl Debug for Http {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Http")
			.field("url", &self.url.as_str())
			.finish_non_exhaustive()
	}
}
