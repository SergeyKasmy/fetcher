/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains all errors that can happen in the (`parent`)[`super`] module

use crate::{
	actions::transforms::field::extract::ExtractError,
	entry::Entry,
	error::{Error, InvalidUrlError, error_trait::BoxErrorWrapper},
};

#[cfg(feature = "action-http")]
use crate::actions::transforms::http::HttpError;

#[cfg(feature = "action-feed")]
use crate::actions::transforms::feed::FeedError;

#[cfg(feature = "action-json")]
use crate::actions::transforms::json::JsonError;

#[cfg(feature = "action-html")]
use crate::actions::transforms::html::HtmlError;

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

	#[cfg(feature = "action-http")]
	#[error("HTTP error")]
	Http(#[from] HttpError),

	#[cfg(feature = "action-feed")]
	#[error("Feed parsing error")]
	Feed(#[from] FeedError),

	#[cfg(feature = "action-html")]
	#[error("HTML parsing error")]
	Html(#[from] HtmlError),

	#[cfg(feature = "action-json")]
	#[error("JSON parsing error")]
	Json(#[from] JsonError),

	#[error("Extraction error")]
	Extract(#[from] ExtractError),

	#[error(transparent)]
	Other(#[from] Box<dyn Error>),
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
#[error("There's nothing to transform from")]
pub struct RawContentsNotSetError;

impl From<Box<dyn StdError + Send + Sync>> for TransformErrorKind {
	fn from(value: Box<dyn StdError + Send + Sync>) -> Self {
		Self::Other(Box::new(BoxErrorWrapper(value)))
	}
}

impl From<Infallible> for TransformErrorKind {
	fn from(value: Infallible) -> Self {
		match value {}
	}
}

#[cfg(feature = "nightly")]
impl From<!> for TransformErrorKind {
	fn from(value: !) -> Self {
		match value {}
	}
}

impl Error for TransformError {
	fn is_network_related(&self) -> Option<&dyn Error> {
		match &self.kind {
			#[cfg(feature = "action-http")]
			TransformErrorKind::Http(HttpError::Other(_)) => Some(self),
			TransformErrorKind::Other(other_err) if other_err.is_network_related().is_some() => {
				Some(self)
			}
			_ => None,
		}
	}
}
