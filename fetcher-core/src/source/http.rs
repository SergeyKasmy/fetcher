/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! HTTP source
//!
//! This module contains the [`Http`] struct, that is a source as well as a transform

use crate::{
	action::transform::result::{TransformResult, TransformedEntry, TransformedMessage},
	entry::Entry,
	error::{source::HttpError, transform::HttpError as HttpTransformError, InvalidUrlError},
	sink::Message,
	utils::OptionExt,
};

use once_cell::sync::OnceCell;
use std::fmt::{Debug, Display};
use url::Url;

const USER_AGENT: &str =
	"Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:96.0) Gecko/20100101 Firefox/96.0";

static CLIENT: OnceCell<reqwest::Client> = OnceCell::new();

/// A source that fetches from the [`URL`](`url`)
pub struct Http {
	/// The URL to fetch from
	pub url: Url,
	request: Request,
	client: reqwest::Client,
}

/// When used as a transform, which field to get the link from?
#[derive(Clone, Copy, Debug)]
pub enum TransformFromField {
	/// The [`Message.link`] field
	MessageLink,
	/// The [`Entry.raw_contents`] field
	RawContents,
}

#[derive(Debug)]
enum Request {
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

	/// Send a request to the [`URL`](`self.url`) and return the result in the [`Entry.raw_contents`] field
	#[tracing::instrument(skip_all)]
	pub async fn get(&self) -> Result<Entry, HttpError> {
		tracing::debug!("Sending an HTTP request");

		let request = match &self.request {
			Request::Get => {
				tracing::trace!("Making an HTTP GET request to {:?}", self.url.as_str());

				self.client.get(self.url.as_str())
			}
			Request::Post(json) => {
				tracing::trace!(
					"Making an HTTP POST request to {:?} with {:#?}",
					self.url.as_str(),
					json
				);

				self.client.post(self.url.as_str()).json(json)
			}
		};

		let response = request
			.header(reqwest::header::USER_AGENT, USER_AGENT)
			.send()
			.await
			.map_err(|e| HttpError::BadRequest(e, self.url.to_string()))?;

		tracing::trace!("Getting text body of the response");
		let page = response
			.text()
			.await
			.map_err(|e| HttpError::BadRequest(e, self.url.to_string()))?;

		// tracing::trace!("Done. Body: ----------------------------------------\n{page:?}\n----------------------------------------\n");
		tracing::trace!("Done");

		Ok(Entry {
			raw_contents: Some(page),
			msg: Message {
				link: Some(self.url.clone()),
				..Default::default()
			},
			..Default::default()
		})
	}

	/// Get the URL from the `entry`, send a GET request to it, and put the result into the [`Entry::raw_contents`] field
	///
	/// # Errors
	/// * if, depending on `from_field`, either [`Message::link`] or [`Entry::raw_contents`] is None
	/// * if the string in the [`Entry::raw_contents`] field when using [`TransformFromField::RawContents`] is not a valid URL
	/// * if there was an error sending the HTTP request
	pub async fn transform(
		entry: &Entry,
		from_field: TransformFromField,
	) -> Result<TransformedEntry, HttpTransformError> {
		let link = match from_field {
			TransformFromField::MessageLink => entry.msg.link.clone(),
			TransformFromField::RawContents => entry.raw_contents.as_ref().try_map(|s| {
				Url::try_from(s.as_str()).map_err(|e| InvalidUrlError(e, s.clone()))
			})?,
		};
		let link = link.ok_or(HttpTransformError::MissingUrl(from_field))?;

		let Entry {
			raw_contents,
			msg: Message { link, .. },
			..
		} = Self::new(link, Request::Get)?.get().await?;

		Ok(TransformedEntry {
			raw_contents: TransformResult::New(raw_contents),
			msg: TransformedMessage {
				link: TransformResult::New(link),
				..Default::default()
			},
			..Default::default()
		})
	}
}

impl Http {
	fn new(url: Url, request: Request) -> Result<Self, HttpError> {
		let client = CLIENT
			.get_or_try_init(|| {
				reqwest::ClientBuilder::new()
					.timeout(std::time::Duration::from_secs(30))
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

impl Debug for Http {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Http")
			.field("url", &self.url.as_str())
			.field("request", &self.request)
			.finish_non_exhaustive()
	}
}

impl Display for TransformFromField {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			TransformFromField::MessageLink => "message's link",
			TransformFromField::RawContents => "raw_contents",
		})
	}
}
