/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Http`] transform that fetches a web page from a link located in a field of the passed [`Entry`]

use async_trait::async_trait;
use reqwest::Client;
use url::Url;

use super::TransformEntry;
use crate::{
	action::transform::{
		field::Field,
		result::{TransformResult, TransformedEntry, TransformedMessage},
	},
	entry::Entry,
	error::InvalidUrlError,
	source::{self, http::HttpError as SourceHttpError, http::Request},
	utils::OptionExt,
};

/// A transform that fetches the page from URL in `from_field` and returns it in [`Entry::raw_contents`]
#[derive(Debug)]
pub struct Http {
	/// The field to get the URL from
	pub from_field: Field,
	client: Client,
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum HttpError {
	#[error("Missing URL in the entry {0} field")]
	MissingUrl(Field),

	#[error("Invalid URL in the entry {0} field")]
	InvalidUrl(Field, #[source] InvalidUrlError),

	#[error(transparent)]
	Other(#[from] crate::source::http::HttpError),
}

impl Http {
	/// Create a new [`Http`] transform
	///
	/// # Errors
	/// This method fails if TLS couldn't be initialized
	pub fn new(from_field: Field) -> Result<Self, SourceHttpError> {
		let client = source::http::CLIENT
			.get_or_try_init(|| {
				reqwest::ClientBuilder::new()
					.timeout(std::time::Duration::from_secs(30))
					.build()
					.map_err(SourceHttpError::TlsInitFailed)
			})?
			.clone();

		Ok(Self { from_field, client })
	}
}

#[async_trait]
impl TransformEntry for Http {
	type Err = HttpError;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		let url: Option<Url> = match self.from_field {
			Field::Title => entry.msg.title.as_deref().try_map(|s| {
				Url::try_from(s).map_err(|e| {
					HttpError::InvalidUrl(self.from_field, InvalidUrlError(e, s.to_owned()))
				})
			})?,
			Field::Body => entry.msg.body.as_deref().try_map(|s| {
				Url::try_from(s).map_err(|e| {
					HttpError::InvalidUrl(self.from_field, InvalidUrlError(e, s.to_owned()))
				})
			})?,
			Field::Link => entry.msg.link.clone(),
			Field::Id => entry.id.as_ref().try_map(|id| {
				Url::try_from(id.0.as_str()).map_err(|e| {
					HttpError::InvalidUrl(self.from_field, InvalidUrlError(e, id.0.clone()))
				})
			})?,
			Field::ReplyTo => entry.reply_to.as_ref().try_map(|id| {
				Url::try_from(id.0.as_str()).map_err(|e| {
					HttpError::InvalidUrl(self.from_field, InvalidUrlError(e, id.0.clone()))
				})
			})?,
			Field::RawContets => entry.raw_contents.as_deref().try_map(|s| {
				Url::try_from(s).map_err(|e| {
					HttpError::InvalidUrl(self.from_field, InvalidUrlError(e, s.to_owned()))
				})
			})?,
		};

		let url = url.ok_or_else(|| HttpError::MissingUrl(self.from_field))?;

		let new_page = source::http::send_request(&self.client, &Request::Get, &url).await?;

		Ok(vec![TransformedEntry {
			raw_contents: TransformResult::New(new_page),
			msg: TransformedMessage {
				link: TransformResult::New(url),
				..Default::default()
			},
			..Default::default()
		}])
	}
}
