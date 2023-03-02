/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! HTTP source
//!
//! This module contains the [`Http`] struct, that is a source as well as a transform

use crate::{entry::Entry, sink::Message, source::error::SourceError};

use async_trait::async_trait;
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::{fmt::Debug, time::Duration};
use url::Url;

use super::Fetch;

const USER_AGENT: &str =
	"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:96.0) Gecko/20100101 Firefox/96.0";

pub(crate) static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();

/// A source that fetches from the [`URL`](`url`)
pub struct Http {
	/// The URL to fetch from
	pub url: Url,
	request: Request,
	client: reqwest::Client,
}

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum HttpError {
	#[error("Invalid JSON for the POST request")]
	BadJson(#[from] serde_json::Error),

	#[error("Failed to init TLS")]
	TlsInitFailed(#[source] reqwest::Error),

	#[error("Can't send an HTTP request to {1:?}")]
	BadRequest(#[source] reqwest::Error, String),
}

#[derive(Debug)]
pub(crate) enum Request {
	Get,
	Post(serde_json::Value),
}

impl Http {
	/// Create a new HTTP client that sends GET requests
	///
	/// # Errors
	/// This method fails if TLS couldn't be initialized
	pub fn new_get(url: Url) -> Result<Self, HttpError> {
		Self::new(url, Request::Get)
	}

	/// Create a new HTTP client that sends POST requests
	///
	/// # Errors
	/// This method fails if body isn't valid JSON or TLS couldn't be initialized
	pub fn new_post(url: Url, body: &str) -> Result<Self, HttpError> {
		Self::new(url, Request::Post(serde_json::from_str(body)?))
	}
}

#[async_trait]
impl Fetch for Http {
	/// Send a request to the [`URL`](`self.url`) and return the result in the [`Entry.raw_contents`] field
	#[tracing::instrument(skip_all)]
	async fn fetch(&mut self) -> Result<Vec<Entry>, SourceError> {
		self.fetch_impl().await.map(|x| vec![x]).map_err(Into::into)
	}
}

impl Http {
	fn new(url: Url, request: Request) -> Result<Self, HttpError> {
		let client = CLIENT
			.get_or_try_init(|| {
				reqwest::ClientBuilder::new()
					.timeout(Duration::from_secs(30))
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

	async fn fetch_impl(&self) -> Result<Entry, HttpError> {
		tracing::debug!("Sending an HTTP request");

		let page = send_request(&self.client, &self.request, &self.url).await?;

		// tracing::trace!("Done. Body: ----------------------------------------\n{page:?}\n----------------------------------------\n");

		Ok(Entry {
			raw_contents: Some(page),
			msg: Message {
				link: Some(self.url.clone()),
				..Default::default()
			},
			..Default::default()
		})
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
