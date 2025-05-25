/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that can happen in the (`parent`)[`super`] module

use crate::{
	actions::transforms::{
		feed::FeedError, field::extract::ExtractError, html::HtmlError, http::HttpError,
		json::JsonError,
	},
	entry::Entry,
	error::InvalidUrlError,
};

use std::{convert::Infallible, error::Error as StdError};

/// An error that occured during transforming of entries
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("Error transforming entry")]
pub struct TransformError {
	#[source]
	pub kind: TransformErrorKind,
	pub original_entry: Entry,
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum TransformErrorKind {
	#[error("Message link is not a valid URL after transforming")]
	FieldLinkTransformInvalidUrl(#[source] InvalidUrlError),

	#[error("HTTP error")]
	Http(#[from] HttpError),

	#[error("Feed parsing error")]
	Feed(#[from] FeedError),

	#[error("HTML parsing error")]
	Html(#[from] HtmlError),

	#[error("JSON parsing error")]
	Json(#[from] JsonError),

	#[error("Extraction error")]
	Extract(#[from] ExtractError),

	#[error("Other error")]
	Other(#[from] Box<dyn StdError + Send + Sync>),
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("There's nothing to transform from")]
pub struct RawContentsNotSetError;

impl From<Infallible> for TransformErrorKind {
	fn from(inf: Infallible) -> Self {
		match inf {}
	}
}

impl TransformError {
	pub(crate) fn is_connection_err(&self) -> Option<&(dyn StdError + Send + Sync)> {
		match &self.kind {
			TransformErrorKind::Http(HttpError::Other(_)) => Some(self),
			_ => None,
		}
	}
}
