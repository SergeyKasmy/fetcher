/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! HTTP source
//!
//! This module contains the [`Http`] struct, that is a source as well as a transform

pub use reqwest;

use crate::{entry::Entry, sinks::message::Message};

use once_cell::sync::OnceCell;
use reqwest::Client;
use std::{convert::identity, fmt::Debug, time::Duration};
use url::Url;

use super::Fetch;

const USER_AGENT: &str =
	"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:96.0) Gecko/20100101 Firefox/96.0";

pub(crate) static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();

pub use serde_json::Value as Json;

/// A source that fetches from the [`URL`](`url`)
pub struct Http {
	/// The URL to fetch from
	pub url: Url,
	request: Request,
	client: reqwest::Client,
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum HttpError {
	#[error("Invalid JSON for the POST request")]
	BadJson(#[from] serde_json::Error),

	#[error("Failed to init TLS")]
	TlsInitFailed(#[source] reqwest::Error),

	#[error("Can't send an HTTP request to {1:?}")]
	BadRequest(#[source] reqwest::Error, String),

	#[error("Not a valid URL")]
	InvalidUrl(#[from] url::ParseError),
}

/// HTTP request type
#[derive(Debug)]
pub enum Request {
	/// HTTP GET request
	Get,

	/// HTTP POST request, contaning a JSON payload
	Post(Json),
}

impl Http {
	/// Create a new HTTP client that sends GET requests
	///
	/// # Errors
	/// This method fails if TLS couldn't be initialized
	pub fn new_get(url: impl TryInto<Url, Error = url::ParseError>) -> Result<Self, HttpError> {
		Self::new_with_client_config(url.try_into()?, Request::Get, identity)
	}

	/// Create a new HTTP client that sends POST requests
	///
	/// # Errors
	/// This method fails if body isn't valid JSON or TLS couldn't be initialized
	pub fn new_post(
		url: impl TryInto<Url, Error = url::ParseError>,
		body: &str,
	) -> Result<Self, HttpError> {
		Self::new_with_client_config(
			url.try_into()?,
			Request::Post(serde_json::from_str(body)?),
			identity,
		)
	}

	/// Creates a new HTTP client with a closure that gets passed a [`reqwest::ClientBuilder`]
	/// to allow more configuration from the caller (e.g. setting headers & cookie stores).
	///
	/// This is the most general constructor for [`Http`] and allows the most freedom.
	///
	/// # Errors
	/// This method fails if TLS couldn't be initialized
	pub fn new_with_client_config<F>(
		url: Url,
		request: Request,
		builder_config: F,
	) -> Result<Self, HttpError>
	where
		F: FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder,
	{
		let client = CLIENT
			.get_or_try_init(|| {
				let builder = reqwest::ClientBuilder::new().timeout(Duration::from_secs(30));

				builder_config(builder)
					.build()
					.map_err(HttpError::TlsInitFailed)
			})?
			.clone();

		Ok(Self {
			url,
			request,
			client,
		})
	}
}

impl Fetch for Http {
	type Err = HttpError;

	/// Send a request to the [`URL`](`self.url`) and return the result in the [`Entry.raw_contents`] field
	#[tracing::instrument(skip_all)]
	async fn fetch(&mut self) -> Result<Vec<Entry>, Self::Err> {
		tracing::debug!("Sending an HTTP request");

		let page = send_request(&self.client, &self.request, &self.url).await?;

		let entry = Entry::builder()
			.raw_contents(page)
			.msg(Message::builder().link(self.url.as_str().to_owned()))
			.build();

		Ok(vec![entry])
	}
}

pub(crate) async fn send_request(
	client: &Client,
	request: &Request,
	url: &Url,
) -> Result<String, HttpError> {
	let request = match request {
		Request::Get => {
			tracing::trace!("Making an HTTP GET request to {:?}", url.as_str());

			client.get(url.as_str())
		}
		Request::Post(json) => {
			tracing::trace!(
				"Making an HTTP POST request to {:?} with {:#?}",
				url.as_str(),
				json
			);

			client.post(url.as_str()).json(json)
		}
	};

	let response = request
		// TODO: move this to builder config and allow the user to override it. There's ClientBuilder::user_agent() I believe
		.header(reqwest::header::USER_AGENT, USER_AGENT)
		.send()
		.await
		.map_err(|e| HttpError::BadRequest(e, url.to_string()))?;

	tracing::trace!("Getting text body of the response");
	response
		.text()
		.await
		.map_err(|e| HttpError::BadRequest(e, url.to_string()))
}

impl Debug for Http {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Http")
			.field("url", &self.url.as_str())
			.field("request", &self.request)
			.finish_non_exhaustive()
	}
}
