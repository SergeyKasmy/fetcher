/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Html`] parser

pub mod error;

pub use self::error::HtmlError;
pub use scraper::Selector;

use self::error::{ErrorLocation, HtmlErrorInner};
use super::Transform;
use crate::{
	StaticStr,
	actions::transforms::{
		error::RawContentsNotSetError,
		result::{OptionUnwrapTransformResultExt, TransformedEntry, TransformedMessage},
	},
	entry::{Entry, EntryId},
	sinks::message::Media,
	utils::OptionExt,
};

use either::Either;
use itertools::Itertools;
use non_non_full::NonEmptyVec;
use scraper::{ElementRef, Html as HtmlDom, error::SelectorErrorKind};
use std::{borrow::Cow, iter};

/// HTML parser
#[derive(bon::Builder, Debug)]
pub struct Html {
	/// One or more query to find the text of an item. If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field
	// TODO: what happens when the option is Some but the vec is empty? Should be handled probs
	#[builder(field)]
	pub text: Option<NonEmptyVec<DataSelector>>,

	/// Selector to find an item/entry/article in a list on the page. None means to thread the entire page as a single item
	#[builder(with = |sel: &str| -> Result<_, SelectorErrorKind> { Selector::parse(sel) })]
	pub item: Option<Selector>,

	/// Selector to find the title of an item
	#[builder(setters(
		name = title_internal,
		vis = ""
	))]
	pub title: Option<DataSelector>,

	/// Selector to find the ID of an item
	#[builder(setters(
		name = id_internal,
		vis = ""
	))]
	pub id: Option<DataSelector>,

	/// Selector to find the link to an item
	#[builder(setters(
		name = link_internal,
		vis = ""
	))]
	pub link: Option<DataSelector>,

	// TODO: support more media types
	// TODO: why only one selector? JSON transform supports many
	/// Selector to find the image of that item
	#[builder(setters(
		name = img_internal,
		vis = ""
	))]
	pub img: Option<DataSelector>,
}

/// A [`Selector`] can only select an HTML element.
/// A [`DataSelector`] makes it possible to specify where the expect data should be, e.g. in an attribute or as the text of the element
#[derive(Clone, Debug)]
pub struct DataSelector {
	/// A CSS selector to find the HTML element
	pub selector: Selector,

	/// Places where to extract the expected data from
	pub locations: Vec<DataLocation>,

	/// If true, don't error if the data wasn't found
	pub optional: bool,
}

/// Location of the data we are looking for in an attribute
#[derive(Clone, Debug)]
pub enum DataLocation {
	/// Text of the element
	Text,

	/// An attribute of the element
	Attribute(StaticStr),
}

impl Transform for Html {
	type Err = HtmlError;

	async fn transform_entry(&mut self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		tracing::trace!("Parsing raw_contents as HTML");

		// TODO: check .errors and .quirks
		let dom =
			HtmlDom::parse_document(entry.raw_contents.as_ref().ok_or(RawContentsNotSetError)?);

		let root = dom.root_element();

		if root.text().collect::<String>().trim().is_empty() {
			tracing::warn!("HTML body is completely empty");

			// TODO: return an error instead
			return Ok(Vec::new());
		}

		let items = match self.item.as_ref() {
			Some(item_sel) => Either::Left(root.select(item_sel)),
			None => Either::Right(iter::once(root)),
		};

		let entries = items
			.map(|item| self.extract_entry(item))
			.collect::<Result<Vec<_>, _>>()?;

		tracing::debug!("Found {} HTML entries total", entries.len());

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
			.try_and_then(|q| extract_title(html_fragment, q))
			.map_err(|error| HtmlError::Inner {
				r#where: ErrorLocation::Title,
				error,
			})?;

		let body = self
			.text
			.as_ref()
			.try_map(|q| extract_body(html_fragment, q))
			.map_err(|(error, index)| HtmlError::Inner {
				r#where: ErrorLocation::Text { index },
				error,
			})?;

		let id = self
			.id
			.as_ref()
			.try_and_then(|q| extract_id(html_fragment, q))
			.map_err(|error| HtmlError::Inner {
				r#where: ErrorLocation::Id,
				error,
			})?;

		let link = self
			.link
			.as_ref()
			.try_and_then(|q| extract_link(html_fragment, q))
			.map_err(|error| HtmlError::Inner {
				r#where: ErrorLocation::Link,
				error,
			})?;

		let imgs = self
			.img
			.as_ref()
			.try_and_then(|q| extract_imgs(html_fragment, q))
			.map_err(|error| HtmlError::Inner {
				r#where: ErrorLocation::Img,
				error,
			})?;

		Ok(TransformedEntry {
			id: id.and_then(EntryId::new).unwrap_or_prev(),
			raw_contents: body.clone().unwrap_or_prev(),
			msg: TransformedMessage {
				title: title.unwrap_or_prev(),
				body: body.unwrap_or_prev(),
				link: link.unwrap_or_prev(),
				media: imgs.unwrap_or_prev(),
			},
			..Default::default()
		})
	}
}

/// Extract data from the provided HTML tags
fn extract_data(
	html_fragment: ElementRef<'_>,
	sel: &DataSelector,
) -> Result<Option<Vec<String>>, HtmlErrorInner> {
	let matched_elements = html_fragment.select(&sel.selector).collect::<Vec<_>>();

	if matched_elements.is_empty() {
		if sel.optional {
			// TODO: add warn
			return Ok(None);
		} else {
			return Err(HtmlErrorInner::SelectorNotMatched(sel.selector.clone()));
		}
	}

	let extracted_data = matched_elements
		.iter()
		.cartesian_product(sel.locations.iter())
		.filter_map(|(elem, location)| {
			let extracted_text = match location {
				DataLocation::Text => Some(Cow::Owned(elem.text().collect::<String>())),
				DataLocation::Attribute(attr) => elem.attr(attr).map(Cow::Borrowed),
			};

			extracted_text.map(|s| s.trim().to_owned())
		})
		.collect::<Vec<_>>();

	// selector matched an element that didn't have a required attribute
	if extracted_data.is_empty() {
		if sel.optional {
			return Ok(None);
		} else {
			return Err(HtmlErrorInner::DataNotFoundInElement(sel.clone()));
		}
	}
	// selector matched an empty (or full of whitespace) element
	else if extracted_data.iter().all(String::is_empty) {
		if sel.optional {
			return Ok(None);
		} else {
			return Err(HtmlErrorInner::ElementEmpty(sel.clone()));
		}
	}

	Ok(Some(extracted_data))
}

fn extract_title(
	html_fragment: ElementRef<'_>,
	selector: &DataSelector,
) -> Result<Option<String>, HtmlErrorInner> {
	Ok(extract_data(html_fragment, selector)?.map(|it| it.join("\n\n"))) // concat string with "\n\n" as sep
}

fn extract_body(
	html_fragment: ElementRef<'_>,
	selectors: &[DataSelector],
) -> Result<String, (HtmlErrorInner, usize)> {
	Ok(selectors
		.iter()
		.enumerate()
		.map(|(sel_idx, selector)| {
			extract_data(html_fragment, selector).map_err(|error| (error, sel_idx))
		})
		.collect::<Result<Vec<_>, _>>()?
		.into_iter()
		.flatten() // flatten options, ignore none's
		.flatten() // flatten inner iterator
		.join("\n\n"))
}

fn extract_id(
	html_fragment: ElementRef<'_>,
	selector: &DataSelector,
) -> Result<Option<String>, HtmlErrorInner> {
	Ok(extract_data(html_fragment, selector)?.map(|v| v.into_iter().collect::<String>())) // concat strings if several
}

fn extract_link(
	html_fragment: ElementRef<'_>,
	selector: &DataSelector,
) -> Result<Option<String>, HtmlErrorInner> {
	let urls = extract_data(html_fragment, selector)?;

	Ok(urls.map(|mut it| it.swap_remove(0)))
}

fn extract_imgs(
	html_fragment: ElementRef<'_>,
	selector: &DataSelector,
) -> Result<Option<NonEmptyVec<Media>>, HtmlErrorInner> {
	let extracted_strings = extract_data(html_fragment, selector)?;

	let as_images = extracted_strings.and_then(|it| {
		let vec = it.into_iter().map(Media::Photo).collect::<Vec<_>>();
		NonEmptyVec::new(vec)
	});

	Ok(as_images)
}

impl<S: html_builder::State> HtmlBuilder<S> {
	/// Selector to find the text of an item.
	///
	/// Can be called multiple times.
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	pub fn text(self, sel: &str) -> Result<Self, SelectorErrorKind> {
		self.text_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// Selector to find the text of an item.
	pub fn text_with_conf(
		mut self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<Self, SelectorErrorKind> {
		let data_selector = DataSelector {
			selector: Selector::parse(sel)?,
			locations: locations.into_iter().collect(),
			optional,
		};

		match &mut self.text {
			Some(text) => text.push(data_selector),
			None => self.text = Some(NonEmptyVec::with_first(data_selector)),
		}

		Ok(self)
	}

	/// Selector to find the title of an item.
	///
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	pub fn title(
		self,
		sel: &str,
	) -> Result<HtmlBuilder<html_builder::SetTitle<S>>, SelectorErrorKind>
	where
		S::Title: html_builder::IsUnset,
	{
		self.title_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// Selector to find the title of an item.
	pub fn title_with_conf(
		self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<HtmlBuilder<html_builder::SetTitle<S>>, SelectorErrorKind>
	where
		S::Title: html_builder::IsUnset,
	{
		Ok(self.title_internal(DataSelector {
			selector: Selector::parse(sel)?,
			locations: locations.into_iter().collect(),
			optional,
		}))
	}

	/// Selector to find the id of an item.
	///
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	pub fn id(self, sel: &str) -> Result<HtmlBuilder<html_builder::SetId<S>>, SelectorErrorKind>
	where
		S::Id: html_builder::IsUnset,
	{
		self.id_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// Selector to find the id of an item.
	pub fn id_with_conf(
		self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<HtmlBuilder<html_builder::SetId<S>>, SelectorErrorKind>
	where
		S::Id: html_builder::IsUnset,
	{
		Ok(self.id_internal(DataSelector {
			selector: Selector::parse(sel)?,
			locations: locations.into_iter().collect(),
			optional,
		}))
	}

	/// Selector to find the link of an item.
	///
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	pub fn link(self, sel: &str) -> Result<HtmlBuilder<html_builder::SetLink<S>>, SelectorErrorKind>
	where
		S::Link: html_builder::IsUnset,
	{
		self.link_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// Selector to find the link of an item.
	pub fn link_with_conf(
		self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<HtmlBuilder<html_builder::SetLink<S>>, SelectorErrorKind>
	where
		S::Link: html_builder::IsUnset,
	{
		Ok(self.link_internal(DataSelector {
			selector: Selector::parse(sel)?,
			locations: locations.into_iter().collect(),
			optional,
		}))
	}

	/// Selector to find the img of an item.
	///
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	pub fn img(self, sel: &str) -> Result<HtmlBuilder<html_builder::SetImg<S>>, SelectorErrorKind>
	where
		S::Img: html_builder::IsUnset,
	{
		self.img_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// Selector to find the img of an item.
	pub fn img_with_conf(
		self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<HtmlBuilder<html_builder::SetImg<S>>, SelectorErrorKind>
	where
		S::Img: html_builder::IsUnset,
	{
		Ok(self.img_internal(DataSelector {
			selector: Selector::parse(sel)?,
			locations: locations.into_iter().collect(),
			optional,
		}))
	}
}
