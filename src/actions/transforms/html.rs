/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Html`] parser

pub mod error;

pub use self::error::HtmlError;
pub use scraper::Selector;

use self::error::{ErrorLocation, HtmlErrorInner, SelectorError};
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
use scraper::{ElementRef, Html as HtmlDom};
use std::{borrow::Cow, iter};

/// HTML parser
#[derive(bon::Builder, Debug)]
pub struct Html {
	/// One or more CSS selectors to find the text of an item.
	/// If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field.
	#[builder(field)]
	pub text: Option<NonEmptyVec<DataSelector>>,

	/// CSS selector to find an item/entry/article in a list on the page. None means to threat the entire page as a single item
	#[builder(with = |sel: &str| -> Result<_, SelectorError> { Selector::parse(sel).map_err(Into::into) })]
	pub item: Option<Selector>,

	/// CSS selector to find the title of an item
	#[builder(setters(
		name = title_internal,
		vis = ""
	))]
	pub title: Option<DataSelector>,

	/// CSS selector to find the ID of an item
	#[builder(setters(
		name = id_internal,
		vis = ""
	))]
	pub id: Option<DataSelector>,

	/// CSS selector to find the link to an item
	#[builder(setters(
		name = link_internal,
		vis = ""
	))]
	pub link: Option<DataSelector>,

	// TODO: support more media types
	// TODO: why only one selector? JSON transform supports many
	/// CSS selector to find the image of that item
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
	/// CSS selector to find the HTML element
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

		// don't check this in prod, it should probably be fine. If it's not, it'll hopefully be caught while developing
		#[cfg(debug_assertions)]
		if root.text().collect::<String>().trim().is_empty() {
			tracing::warn!("HTML body is empty");
		}

		let items = match self.item.as_ref() {
			Some(item_sel) => {
				let matched_items = root.select(item_sel);

				if matched_items.clone().count() == 0 {
					tracing::error!("Item selector didn't match an item");
					return Err(HtmlError::Inner {
						error: HtmlErrorInner::SelectorNotMatched(item_sel.clone()),
						r#where: ErrorLocation::Item,
					});
				}

				Either::Left(matched_items)
			}
			None => Either::Right(iter::once(root)),
		};

		let entries = items
			.map(|item| self.extract_entry(item))
			.collect::<Result<Vec<_>, _>>()?;

		tracing::debug!("Found {} HTML entries total", entries.len());

		Ok(entries)
	}
}

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
			tracing::debug!("Selector didn't match any element");
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
			tracing::debug!("Selector matched an element but wasn't able to extract anything");
			return Ok(None);
		} else {
			return Err(HtmlErrorInner::DataNotFoundInElement(sel.clone()));
		}
	}
	// selector matched an empty (or full of whitespace) element
	else if extracted_data.iter().all(String::is_empty) {
		if sel.optional {
			tracing::debug!("Selector matched an element but its contents were empty");
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
	/// CSS Selector to find the text of an item.
	///
	/// Can be called multiple times.
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn text(self, sel: &str) -> Result<Self, SelectorError> {
		self.text_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// CSS Selector to find the text of an item.
	///
	/// Can be called multiple times.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn text_with_conf(
		mut self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<Self, SelectorError> {
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

	/// CSS Selector to find the title of an item.
	///
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn title(self, sel: &str) -> Result<HtmlBuilder<html_builder::SetTitle<S>>, SelectorError>
	where
		S::Title: html_builder::IsUnset,
	{
		self.title_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// CSS Selector to find the title of an item.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn title_with_conf(
		self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<HtmlBuilder<html_builder::SetTitle<S>>, SelectorError>
	where
		S::Title: html_builder::IsUnset,
	{
		Ok(self.title_internal(DataSelector {
			selector: Selector::parse(sel)?,
			locations: locations.into_iter().collect(),
			optional,
		}))
	}

	/// CSS Selector to find the ID of an item.
	///
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn id(self, sel: &str) -> Result<HtmlBuilder<html_builder::SetId<S>>, SelectorError>
	where
		S::Id: html_builder::IsUnset,
	{
		self.id_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// CSS Selector to find the ID of an item.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn id_with_conf(
		self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<HtmlBuilder<html_builder::SetId<S>>, SelectorError>
	where
		S::Id: html_builder::IsUnset,
	{
		Ok(self.id_internal(DataSelector {
			selector: Selector::parse(sel)?,
			locations: locations.into_iter().collect(),
			optional,
		}))
	}

	/// CSS Selector to find the link of an item.
	///
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn link(self, sel: &str) -> Result<HtmlBuilder<html_builder::SetLink<S>>, SelectorError>
	where
		S::Link: html_builder::IsUnset,
	{
		self.link_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// CSS Selector to find the link of an item.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn link_with_conf(
		self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<HtmlBuilder<html_builder::SetLink<S>>, SelectorError>
	where
		S::Link: html_builder::IsUnset,
	{
		Ok(self.link_internal(DataSelector {
			selector: Selector::parse(sel)?,
			locations: locations.into_iter().collect(),
			optional,
		}))
	}

	/// CSS Selector to find the image of an item.
	///
	/// Makes it not optional and extract from [`DataLocation::Text`] by default.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn img(self, sel: &str) -> Result<HtmlBuilder<html_builder::SetImg<S>>, SelectorError>
	where
		S::Img: html_builder::IsUnset,
	{
		self.img_with_conf(sel, iter::once(DataLocation::Text), false)
	}

	/// CSS Selector to find the image of an item.
	///
	/// # Errors
	/// if the CSS selector `sel` isn't actually a valid CSS selector
	pub fn img_with_conf(
		self,
		sel: &str,
		locations: impl IntoIterator<Item = DataLocation>,
		optional: bool,
	) -> Result<HtmlBuilder<html_builder::SetImg<S>>, SelectorError>
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

// TODO: add tests for multiple text selectors and multiple data locations with a "check" function to remove code duplication
#[cfg(test)]
mod tests {
	use std::{iter, sync::LazyLock};

	use assert_matches::assert_matches;

	use super::Html;
	use crate::{
		actions::transforms::{
			Transform,
			html::{
				DataLocation, HtmlError,
				error::{ErrorLocation, HtmlErrorInner},
			},
		},
		entry::Entry,
	};

	const DUMMY_HTML_PAGE: &str = r#"
	<body>
		<div>
			<p class="title">Hello, World!</p>
			<a href="https://example.com">Go to example.com</a>
			<h1 class="empty"></h1>
		</div>
	</body>	
	"#;

	static ENTRY: LazyLock<Entry> = LazyLock::new(|| {
		Entry::builder()
			.raw_contents(DUMMY_HTML_PAGE.to_owned())
			.build()
	});

	#[tokio::test]
	async fn item_found() {
		Html::builder()
			.item("div")
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await
			.unwrap();
	}

	#[tokio::test]
	async fn item_not_found() {
		let result = Html::builder()
			.item("article")
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await;

		assert_matches!(
			result,
			Err(HtmlError::Inner {
				r#where: ErrorLocation::Item,
				error: HtmlErrorInner::SelectorNotMatched(_)
			})
		);
	}

	#[tokio::test]
	async fn title_extracts_by_full_path() {
		let transformed_entries = Html::builder()
			.item("div")
			.unwrap()
			.title("p.title")
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await
			.unwrap();

		let mut transformed_entries = transformed_entries.into_iter();
		assert_eq!(
			transformed_entries
				.next()
				.unwrap()
				.into_entry(&*ENTRY)
				.msg
				.title
				.as_deref(),
			Some("Hello, World!")
		);
		assert_matches!(transformed_entries.next(), None);
	}

	#[tokio::test]
	async fn title_extracts_by_class() {
		let transformed_entries = Html::builder()
			.item("div")
			.unwrap()
			.title(".title")
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await
			.unwrap();

		let mut transformed_entries = transformed_entries.into_iter();
		assert_eq!(
			transformed_entries
				.next()
				.unwrap()
				.into_entry(&*ENTRY)
				.msg
				.title
				.as_deref(),
			Some("Hello, World!")
		);
		assert_matches!(transformed_entries.next(), None);
	}

	#[tokio::test]
	async fn title_extracts_by_element_type() {
		let transformed_entries = Html::builder()
			.item("div")
			.unwrap()
			.title("p")
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await
			.unwrap();

		let mut transformed_entries = transformed_entries.into_iter();
		assert_eq!(
			transformed_entries
				.next()
				.unwrap()
				.into_entry(&*ENTRY)
				.msg
				.title
				.as_deref(),
			Some("Hello, World!")
		);
		assert_matches!(transformed_entries.next(), None);
	}

	#[tokio::test]
	async fn title_not_found() {
		let result = Html::builder()
			.item("div")
			.unwrap()
			.title("article.title")
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await;

		assert_matches!(
			result,
			Err(HtmlError::Inner {
				r#where: ErrorLocation::Title,
				error: HtmlErrorInner::SelectorNotMatched(_)
			})
		);
	}

	#[tokio::test]
	async fn link_extracts_from_attr() {
		let transformed_entries = Html::builder()
			.item("div")
			.unwrap()
			.link_with_conf(
				"a",
				iter::once(DataLocation::Attribute("href".into())),
				false,
			)
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await
			.unwrap();

		let mut transformed_entries = transformed_entries.into_iter();
		assert_eq!(
			transformed_entries
				.next()
				.unwrap()
				.into_entry(&*ENTRY)
				.msg
				.link
				.as_deref(),
			Some("https://example.com")
		);
		assert_matches!(transformed_entries.next(), None);
	}

	#[tokio::test]
	async fn link_matched_but_wrong_attr() {
		let result = Html::builder()
			.item("div")
			.unwrap()
			.link_with_conf(
				"a",
				iter::once(DataLocation::Attribute("url".into())),
				false,
			)
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await;

		assert_matches!(
			result,
			Err(HtmlError::Inner {
				r#where: ErrorLocation::Link,
				error: HtmlErrorInner::DataNotFoundInElement(_),
			})
		);
	}

	#[tokio::test]
	async fn body_matched_empty_elem() {
		let result = Html::builder()
			.item("div")
			.unwrap()
			.text("h1.empty")
			.unwrap()
			.build()
			.transform_entry(ENTRY.clone())
			.await;

		assert_matches!(
			result,
			Err(HtmlError::Inner {
				r#where: ErrorLocation::Text { index: 0 },
				error: HtmlErrorInner::ElementEmpty(_),
			})
		);
	}
}
