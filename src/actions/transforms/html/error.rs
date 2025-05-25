/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`HtmlError`] and [`ErrorLocation`] types

use std::fmt::Display;

use scraper::{Selector, selector::ToCss};

use super::DataSelector;
use crate::actions::transforms::error::RawContentsNotSetError;

/// An error that occured during parsing the HTML tree
#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum HtmlError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error("Unable to get the {}", .r#where)]
	Inner {
		r#where: ErrorLocation,
		#[source]
		error: HtmlErrorInner,
	},
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum HtmlErrorInner {
	#[error("Selector {} didn't match anything", .0.to_css_string())]
	SelectorNotMatched(Selector),

	#[error("Data not found in element selected by {} in {:?}", .0.selector.to_css_string(), .0.locations)]
	DataNotFoundInElement(DataSelector),

	#[error("HTML element at {} ({:?}) is empty", .0.selector.to_css_string(), .0.locations)]
	ElementEmpty(DataSelector),
}

/// The error occured while parsing which field?
#[expect(missing_docs, reason = "self-explanatory")]
#[derive(Clone, Copy, Debug)]
pub enum ErrorLocation {
	Item,
	Title,
	/// `index` contains the index of the selector in the [`Html::text`](`super::Html::text`) array
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
