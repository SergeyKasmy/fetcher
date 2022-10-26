/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Html`] parser as well as a way to query an HTML tag via [`QueryData`]

pub mod query;

use self::query::{DataLocation, ElementDataQuery, ElementKind, ElementQuery};
use super::TransformEntry;
use crate::{
	action::transform::result::{TransformResult as TrRes, TransformedEntry, TransformedMessage},
	entry::Entry,
	error::{
		transform::{HtmlError, RawContentsNotSetError},
		InvalidUrlError,
	},
	sink::Media,
	utils::OptionExt,
};

use either::Either;
use itertools::Itertools;
use soup_kuchiki::{Handle as HtmlNode, NodeExt, QueryBuilderExt, Soup};
use std::iter;
use url::Url;

/// HTML parser
#[derive(Debug)]
pub struct Html {
	/// Query to find an item/entry/article in a list on the page. None means to thread the entire page as a single item
	pub itemq: Option<Vec<ElementQuery>>,
	/// Query to find the title of an item
	pub titleq: Option<ElementDataQuery>,
	/// One or more query to find the text of an item. If more than one, then they all get joined with "\n\n" in-between and put into the [`Message.body`] field
	pub textq: Option<Vec<ElementDataQuery>>, // allow to find multiple paragraphs and join them together
	/// Query to find the id of an item
	pub idq: Option<ElementDataQuery>,
	/// Query to find the link to an item
	pub linkq: Option<ElementDataQuery>,
	/// Query to find the image of that item
	pub imgq: Option<ElementDataQuery>,
}

impl TransformEntry for Html {
	type Error = HtmlError;

	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
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

		let items = match self.itemq.as_ref() {
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
			.titleq
			.as_ref()
			.try_and_then(|q| extract_title(html, q))?;

		let body = self.textq.as_ref().try_map(|q| extract_body(html, q))?;
		let id = self.idq.as_ref().try_and_then(|q| extract_id(html, q))?;

		let link = self
			.linkq
			.as_ref()
			.try_and_then(|q| extract_url(html, q))?
			.try_map(|mut x| {
				x.next()
					.expect("iterator shouldn't be empty, otherwise it would've been None before")
			})?;

		let img = self.imgq.as_ref().try_and_then(|q| extract_imgs(html, q))?;

		Ok(TransformedEntry {
			id: TrRes::Old(id),
			raw_contents: TrRes::Old(body.clone()),
			msg: TransformedMessage {
				title: TrRes::Old(title),
				body: TrRes::Old(body),
				link: TrRes::Old(link),
				media: TrRes::Old(img),
			},
		})
	}
}

/// Extract data from the provided HTML tags
fn extract_data<'a>(
	html: &HtmlNode,
	data_query: &'a ElementDataQuery,
) -> Result<Option<impl Iterator<Item = String> + 'a>, HtmlError> {
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
) -> Result<Option<impl Iterator<Item = Result<Url, HtmlError>> + 'a>, HtmlError> {
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
#[allow(clippy::needless_pass_by_value)]
fn find(html: HtmlNode, elem_query: &ElementQuery) -> impl Iterator<Item = HtmlNode> + '_ {
	match &elem_query.kind {
		ElementKind::Tag(val) => html.tag(val.as_str()).find_all(),
		ElementKind::Class(val) => html.class(val.as_str()).find_all(),
		ElementKind::Attr { name, value } => html.attr(name.as_str(), value.as_str()).find_all(),
	}
	.filter(move |found| {
		if let Some(ignore) = &elem_query.ignore {
			for i in ignore.iter() {
				let should_be_ignored = match i {
					ElementKind::Tag(tag) => found.name() == tag,
					ElementKind::Class(class) => found.get("class").map_or(false, |c| &c == class),
					ElementKind::Attr { name, value } => {
						found.get(name).map_or(false, |a| &a == value)
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

/*
// TODO: rewrite the entire fn to use today/Yesterday words and date formats from the config per source and not global
fn parse_pretty_date(mut date_str: &str) -> Result<DateTime<Utc>, HtmlError> {
	enum DateTimeKind {
		Today,
		Yesterday,
		Other,
	}

	// TODO: properly parse different languages
	const TODAY_WORDS: &[&str] = &["Today", "Heute", "Сегодня"];
	const YESTERDAY_WORDS: &[&str] = &["Yesterday", "Gestern", "Вчера"];

	date_str = date_str.trim();
	let mut datetime_kind = DateTimeKind::Other;

	for w in TODAY_WORDS {
		if date_str.starts_with(w) {
			date_str = &date_str[w.len()..];
			datetime_kind = DateTimeKind::Today;
		}
	}

	for w in YESTERDAY_WORDS {
		if date_str.starts_with(w) {
			date_str = &date_str[w.len()..];
			datetime_kind = DateTimeKind::Yesterday;
		}
	}

	date_str = date_str.trim_matches(',').trim();

	Ok(match datetime_kind {
		DateTimeKind::Today => {
			let time = NaiveTime::parse_from_str(date_str, "%H:%M")?;
			Local::today().and_time(time).unwrap().into() // unwrap NOTE: no idea why it returns an option so I'll just assume it's safe and hope for the best
		}
		DateTimeKind::Yesterday => {
			let time = NaiveTime::parse_from_str(date_str, "%H:%M")?;
			Local::today().pred().and_time(time).unwrap().into() // unwrap NOTE: same as above
		}
		DateTimeKind::Other => Utc
			.from_local_datetime(&NaiveDate::parse_from_str(date_str, "%d.%m.%Y")?.and_hms(0, 0, 0))
			.unwrap(), // unwrap NOTE: same as above
	})
}
*/
