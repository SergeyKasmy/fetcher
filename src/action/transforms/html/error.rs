use std::fmt::Display;

use scraper::{Selector, selector::ToCss};

use crate::action::transforms::error::RawContentsNotSetError;

use super::DataSelector;

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

#[derive(Clone, Copy, Debug)]
pub enum ErrorLocation {
	Item,
	Title,
	Text { index: usize },
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
