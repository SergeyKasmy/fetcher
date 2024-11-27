/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Html`] parser as well as a way to query an HTML tag via [`ElementQuery`]

pub mod query;

use self::query::{
	DataLocation, ElementDataQuery, ElementKind, ElementQuery, ElementQuerySliceExt,
};
use super::TransformEntry;
use crate::{
	action::transform::{
		error::RawContentsNotSetError,
		result::{OptionUnwrapTransformResultExt, TransformedEntry, TransformedMessage},
	},
	entry::Entry,
	error::InvalidUrlError,
	sink::message::Media,
	utils::OptionExt,
};

use async_trait::async_trait;
use either::Either;
use itertools::Itertools;
use soup_kuchiki::{Handle as HtmlNode, NodeExt, QueryBuilderExt, Soup};
use std::iter;
use url::Url;

/// HTML parser
#[derive(Debug)]
pub struct Html {
	/// Query to find an item/entry/article in a list on the page. None means to thread the entire page as a single item
	pub item: Option<Vec<ElementQuery>>,

	/// Query to find the title of an item
	pub title: Option<ElementDataQuery>,

	/// One or more query to find the text of an item. If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field
	pub text: Option<Vec<ElementDataQuery>>, // allow to find multiple paragraphs and join them together

	/// Query to find the id of an item
	pub id: Option<ElementDataQuery>,

	/// Query to find the link to an item
	pub link: Option<ElementDataQuery>,

	/// Query to find the image of that item
	pub img: Option<ElementDataQuery>,
}

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum HtmlError {
	#[error(transparent)]
	RawContentsNotSet(#[from] RawContentsNotSetError),

	#[error("HTML element #{} not found. From query list: \n{}",
			.num + 1,
			.elem_list.display()
			)]
	ElementNotFound {
		num: usize,
		elem_list: Vec<ElementQuery>,
	},

	#[error("Data not found at {data:?} in element fount at {}",
			.element.display())]
	DataNotFoundInElement {
		data: DataLocation,
		element: Vec<ElementQuery>,
	},

	#[error("HTML element {0:?} is empty")]
	ElementEmpty(Vec<ElementQuery>),

	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),
}

#[async_trait]
impl TransformEntry for Html {
	type Err = HtmlError;

	async fn transform_entry(&self, entry: Entry) -> Result<Vec<TransformedEntry>, Self::Err> {
		tracing::debug!("Parsing HTML");

		let dom =
			Soup::new(entry.raw_contents.as_ref().ok_or(RawContentsNotSetError)?).get_handle();

		let body = dom
			.tag("body") // use body as the root html node
			.find()
			.unwrap_or_else(|| {
				tracing::debug!("HTML doesn't contain a body, using the root as the body");

				// or use the entire html if it doesn't exist for some reason (I don't think it should?)
				dom
			});

		if body.text().trim().is_empty() {
			tracing::warn!("HTML body is completely empty");

			return Ok(Vec::new());
		}

		let items = match self.item.as_ref() {
			Some(itemq) => Either::Left(
				find_chain(&body, itemq)
					.map_err(|i| HtmlError::ElementNotFound {
						num: i,
						elem_list: itemq.clone(),
					})?
					.into_iter(),
			),
			None => Either::Right(iter::once(body)),
		};

		let entries = items
			.map(|item| self.extract_entry(&item))
			.collect::<Result<Vec<_>, _>>()?;

		tracing::debug!("Found {num} HTML articles total", num = entries.len());

		Ok(entries)
	}
}

// TODO: make sure (and add tests!) that it errors if no item was found
// Won't remove this one till I add these goddamned tests >:(
impl Html {
	fn extract_entry(&self, html: &HtmlNode) -> Result<TransformedEntry, HtmlError> {
		let title = self
			.title
			.as_ref()
			.try_and_then(|q| extract_title(html, q))?;

		let body = self.text.as_ref().try_map(|q| extract_body(html, q))?;
		let id = self.id.as_ref().try_and_then(|q| extract_id(html, q))?;

		let link = self
			.link
			.as_ref()
			.try_and_then(|q| extract_url(html, q))?
			.try_map(|mut x| {
				x.next()
					.expect("iterator shouldn't be empty, otherwise it would've been None before")
			})?;

		let img = self.img.as_ref().try_and_then(|q| extract_imgs(html, q))?;

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
	html: &HtmlNode,
	data_query: &'a ElementDataQuery,
) -> Result<Option<impl Iterator<Item = String> + use<'a>>, HtmlError> {
	let data = find_chain(html, &data_query.query).map(|nodes| {
		nodes
			.into_iter()
			.map(|html| {
				match &data_query.data_location {
					DataLocation::Text => Some(html.text()),
					DataLocation::Attr(v) => html.get(v),
				}
				.map(|s| s.trim().to_owned())
			})
			.collect::<Option<Vec<_>>>()
	});

	let data = match data {
		Ok(Some(v)) => v,
		Ok(None) | Err(_) if data_query.optional => return Ok(None),
		Ok(None) => {
			return Err(HtmlError::DataNotFoundInElement {
				data: data_query.data_location.clone(),
				element: data_query.query.clone(),
			});
		}
		Err(i) => {
			return Err(HtmlError::ElementNotFound {
				num: i,
				elem_list: data_query.query.clone(),
			});
		}
	};

	if data.iter().all(String::is_empty) {
		return if data_query.optional {
			Ok(None)
		} else {
			Err(HtmlError::ElementEmpty(data_query.query.clone()))
		};
	}

	Ok(Some(data.into_iter().map(|s| match &data_query.regex {
		Some(r) => r.replace(&s).into_owned(),
		None => s,
	})))
}

fn extract_title(
	html: &HtmlNode,
	data_query: &ElementDataQuery,
) -> Result<Option<String>, HtmlError> {
	Ok(extract_data(html, data_query)?.map(|mut it| it.join("\n\n"))) // concat string with "\n\n" as sep
}

fn extract_body(html: &HtmlNode, data_queries: &[ElementDataQuery]) -> Result<String, HtmlError> {
	Ok(data_queries
		.iter()
		.map(|query| extract_data(html, query))
		.collect::<Result<Vec<_>, _>>()?
		.into_iter()
		.flatten() // flatten options, ignore none's
		.flatten() // flatten inner iterator
		.join("\n\n"))
}

fn extract_id(html: &HtmlNode, data_query: &ElementDataQuery) -> Result<Option<String>, HtmlError> {
	Ok(extract_data(html, data_query)?.map(Iterator::collect)) // concat strings if several
}

fn extract_url<'a>(
	html: &HtmlNode,
	query: &'a ElementDataQuery,
) -> Result<Option<impl Iterator<Item = Result<Url, HtmlError>> + use<'a>>, HtmlError> {
	Ok(extract_data(html, query)?.map(|it| {
		it.map(|url| Url::try_from(url.as_str()).map_err(|e| InvalidUrlError(e, url).into()))
	}))
}

fn extract_imgs(
	html: &HtmlNode,
	data_query: &ElementDataQuery,
) -> Result<Option<Vec<Media>>, HtmlError> {
	extract_url(html, data_query)?.try_map(|it| {
		it.map(|url| url.map(Media::Photo))
			.collect::<Result<Vec<_>, _>>()
	})
}

/// Find all elements matching the query in all the provided HTML parts
///
/// # Errors
/// Errors if the element in the `elem_queries` list wasn't found and returns the id of the query that wasn't found
fn find_chain(html: &HtmlNode, elem_queries: &[ElementQuery]) -> Result<Vec<HtmlNode>, usize> {
	if elem_queries.is_empty() {
		return Ok(vec![html.get_handle()]);
	}

	let mut html_nodes = vec![html.get_handle()];

	for (i, elem_query) in elem_queries.iter().enumerate() {
		html_nodes = html_nodes
			.into_iter()
			.flat_map(|html| find(html, elem_query))
			.collect(); // can't avoid this collect, using iterators directly produces "infinite cycle error"

		if html_nodes.is_empty() {
			return Err(i);
		}
	}

	Ok(html_nodes)
}

/// Find items matching the query in the provided HTML part
// I'm pretty sure this shouldn't capture from elem_query, so this is probably a bug in Soup
#[expect(
	clippy::needless_pass_by_value,
	reason = "HtmlNode is already just a pointer"
)]
fn find(html: HtmlNode, elem_query: &ElementQuery) -> impl Iterator<Item = HtmlNode> {
	match &elem_query.kind {
		ElementKind::Tag(val) => html.tag(val.as_str()).find_all(),
		ElementKind::Class(val) => html.class(val.as_str()).find_all(),
		ElementKind::Attr { name, value } => html.attr(name.as_str(), value.as_str()).find_all(),
	}
	.filter(move |found| {
		if let Some(ignore) = &elem_query.ignore {
			for i in ignore {
				let should_be_ignored = match i {
					ElementKind::Tag(tag) => found.name() == tag,
					ElementKind::Class(class) => found.get("class").is_some_and(|c| &c == class),
					ElementKind::Attr { name, value } => {
						found.get(name).is_some_and(|a| &a == value)
					}
				};

				if should_be_ignored {
					return false;
				}
			}
		}

		true
	})
}
