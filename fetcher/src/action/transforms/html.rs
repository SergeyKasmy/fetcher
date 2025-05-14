/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Html`] parser as well as a way to query an HTML tag via [`ElementQuery`]
// TODO: cleanup and update docs

use super::Transform;
use crate::{
	StaticStr,
	action::transforms::{
		error::RawContentsNotSetError,
		result::{OptionUnwrapTransformResultExt, TransformedEntry, TransformedMessage},
	},
	entry::Entry,
	error::InvalidUrlError,
	sinks::message::Media,
	utils::OptionExt,
};

use either::Either;
use itertools::Itertools;
use scraper::{ElementRef, Html as HtmlDom, error::SelectorErrorKind, selector::ToCss};
use std::{borrow::Cow, iter};

pub use scraper::Selector;

// TODO: update doc
/// HTML parser
#[derive(bon::Builder, Debug)]
pub struct Html {
	/// One or more query to find the text of an item. If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field
	// TODO: what happens when the option is Some but the vec is empty? Should be handled probs
	#[builder(field)]
	pub text: Option<Vec<DataSelector>>,

	/// Query to find an item/entry/article in a list on the page. None means to thread the entire page as a single item
	#[builder(with = |sel: &str| -> Result<_, SelectorErrorKind> { Selector::parse(sel) })]
	pub item: Option<Selector>,

	/// Query to find the title of an item
	#[builder(with = |sel: &str, loc: DataLocation| -> Result<_, SelectorErrorKind> { Ok(DataSelector{ selector: Selector::parse(sel)?, location: loc, optional: false })})]
	pub title: Option<DataSelector>,

	/// Query to find the id of an item
	#[builder(with = |sel: &str, loc: DataLocation| -> Result<_, SelectorErrorKind> { Ok(DataSelector{ selector: Selector::parse(sel)?, location: loc, optional: false })})]
	pub id: Option<DataSelector>,

	/// Query to find the link to an item
	// FIXME: make post-op optional
	#[builder(with = |sel: &str, loc: DataLocation | -> Result<_, SelectorErrorKind> { Ok(DataSelector{ selector: Selector::parse(sel)?, location: loc, optional: false })})]
	pub link: Option<DataSelector>,

	/// Query to find the image of that item
	#[builder(with = |sel: &str, loc: DataLocation| -> Result<_, SelectorErrorKind> { Ok(DataSelector{ selector: Selector::parse(sel)?, location: loc, optional: false })})]
	pub img: Option<DataSelector>,
}

#[derive(Clone, Debug)]
pub struct DataSelector {
	pub selector: Selector,
	pub location: DataLocation,
	pub optional: bool,
}

#[derive(Clone, Debug)]
pub enum DataLocation {
	Text,
	Attribute(StaticStr),
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum HtmlError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error("Selector {} didn't match anything", .0.to_css_string())]
	SelectorNotMatched(Selector),

	#[error("Data not found in element selected by {} in {:?}", .0.selector.to_css_string(), .0.location)]
	DataNotFoundInElement(DataSelector),

	#[error("HTML element at {} ({:?}) is empty", .0.selector.to_css_string(), .0.location)]
	ElementEmpty(DataSelector),

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),
}

impl Transform for Html {
	type Err = HtmlError;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		tracing::debug!("Parsing HTML");

		// TODO: check .errors and .quirks
		let dom =
			HtmlDom::parse_document(entry.raw_contents.as_ref().ok_or(RawContentsNotSetError)?);

		let root = dom.root_element();

		if root.text().collect::<String>().trim().is_empty() {
			tracing::warn!("HTML body is completely empty");

			return Ok(Vec::new());
		}

		let items = match self.item.as_ref() {
			Some(item_sel) => Either::Left(root.select(item_sel)),
			None => Either::Right(iter::once(root)),
		};

		let entries = items
			.map(|item| self.extract_entry(item))
			.collect::<Result<Vec<_>, _>>()?;

		tracing::debug!("Found {num} HTML articles total", num = entries.len());

		Ok(entries)
	}
}

// TODO: make sure (and add tests!) that it errors if no item was found
// Won't remove this one till I add these goddamned tests >:(
impl Html {
	fn extract_entry(&self, html_fragment: ElementRef<'_>) -> Result<TransformedEntry, HtmlError> {
		let title = self
			.title
			.as_ref()
			.try_and_then(|q| extract_title(html_fragment, q))?;

		let body = self
			.text
			.as_ref()
			.try_map(|q| extract_body(html_fragment, q))?;

		let id = self
			.id
			.as_ref()
			.try_and_then(|q| extract_id(html_fragment, q))?;

		let link = self
			.link
			.as_ref()
			.try_and_then(|q| extract_url(html_fragment, q))?;

		let img = self
			.img
			.as_ref()
			.try_and_then(|q| extract_imgs(html_fragment, q))?;

		Ok(TransformedEntry {
			id: id.map(Into::into).unwrap_or_prev(),
			raw_contents: body.clone().unwrap_or_prev(),
			msg: TransformedMessage {
				title: title.unwrap_or_prev(),
				body: body.unwrap_or_prev(),
				link: link.unwrap_or_prev(),
				media: img.unwrap_or_prev(),
			},
			..Default::default()
		})
	}
}

/// Extract data from the provided HTML tags
fn extract_data<'a>(
	html_fragment: ElementRef<'a>,
	sel: &DataSelector,
) -> Result<Option<Vec<String>>, HtmlError> {
	let data = html_fragment
		.select(&sel.selector)
		.into_iter()
		.map(|elem| {
			let extracted_text = match &sel.location {
				DataLocation::Text => Some(Cow::Owned(elem.text().collect::<String>())),
				DataLocation::Attribute(attr) => elem.attr(attr).map(Cow::Borrowed),
			};

			extracted_text.map(|s| s.trim().to_owned())
		})
		.collect::<Option<Vec<_>>>();

	let data = match data {
		Some(v) if v.is_empty() => return Err(HtmlError::SelectorNotMatched(sel.selector.clone())),
		Some(v) if v.iter().all(String::is_empty) => {
			return Err(HtmlError::ElementEmpty(sel.clone()));
		}
		Some(v) => v,
		None if sel.optional => return Ok(None),
		None => return Err(HtmlError::DataNotFoundInElement(sel.clone())),
	};

	Ok(Some(data))
}

fn extract_title(
	html_fragment: ElementRef<'_>,
	selector: &DataSelector,
) -> Result<Option<String>, HtmlError> {
	Ok(extract_data(html_fragment, selector)?.map(|it| it.join("\n\n"))) // concat string with "\n\n" as sep
}

fn extract_body(
	html_fragment: ElementRef<'_>,
	selectors: &[DataSelector],
) -> Result<String, HtmlError> {
	Ok(selectors
		.iter()
		.map(|query| extract_data(html_fragment, query))
		.collect::<Result<Vec<_>, _>>()?
		.into_iter()
		.flatten() // flatten options, ignore none's
		.flatten() // flatten inner iterator
		.join("\n\n"))
}

fn extract_id(
	html_fragment: ElementRef<'_>,
	selector: &DataSelector,
) -> Result<Option<String>, HtmlError> {
	Ok(extract_data(html_fragment, selector)?.map(|v| v.into_iter().collect::<String>())) // concat strings if several
}

fn extract_url<'a>(
	html_fragment: ElementRef<'a>,
	selector: &DataSelector,
) -> Result<Option<String>, HtmlError> {
	Ok(extract_data(html_fragment, selector)?.map(|mut it| it.swap_remove(0)))
}

fn extract_imgs(
	html_fragment: ElementRef<'_>,
	selector: &DataSelector,
) -> Result<Option<Vec<Media>>, HtmlError> {
	Ok(extract_data(html_fragment, selector)?
		.map(|it| it.into_iter().map(Media::Photo).collect::<Vec<_>>()))
}

impl<S: html_builder::State> HtmlBuilder<S> {
	pub fn text(mut self, sel: &str, location: DataLocation) -> Result<Self, SelectorErrorKind> {
		self.text.get_or_insert_default().push(DataSelector {
			selector: Selector::parse(sel)?,
			location,
			optional: false,
		});

		Ok(self)
	}
}
