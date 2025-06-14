/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`JsonError`] and [`ErrorLocation`] types

use std::fmt::Display;

use crate::{actions::transforms::error::RawContentsNotSetError, error::InvalidUrlError};

use super::JsonPointer;

/// An error that occured during parsing the JSON tree
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum JsonError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error("Invalid JSON")]
	Invalid(#[from] serde_json::error::Error),

	#[error("Unable to get the {}", .r#where)]
	Inner {
		r#where: ErrorLocation,
		#[source]
		error: JsonErrorInner,
	},
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum JsonErrorInner {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error("JSON key not found. Pointer: {}", pointer.0)]
	KeyNotFound { pointer: JsonPointer },

	#[error("JSON key {pointer:?} is of wrong type: expected {expected_type}, found {found_type}")]
	KeyWrongType {
		pointer: JsonPointer,
		expected_type: &'static str,
		found_type: String,
	},

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),
}

/// The error occured while parsing which field?
// TODO: this is identical to html::error::ErrorLocation. Should this be merged?
#[expect(missing_docs, reason = "self-explanatory")]
#[derive(Clone, Copy, Debug)]
pub enum ErrorLocation {
	Item,
	Title,
	/// `index` contains the index of the selector in the [`Json::text`](`super::Json::text`) array
	Text {
		index: usize,
	},
	Id,
	Link,
	Img,
}

impl Display for ErrorLocation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match *self {
			Self::Item => f.write_str("item"),
			Self::Title => f.write_str("title"),
			Self::Text { index } => write!(f, "text:{index}"),
			Self::Id => f.write_str("id"),
			Self::Link => f.write_str("link"),
			Self::Img => f.write_str("img"),
		}
	}
}
