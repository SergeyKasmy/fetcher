/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Html`] parser as well as a way to query an HTML tag via [`QueryData`]
// TODO: better handle invalid config values
// TODO: make sure read_filter_type not_present_in_read_list only works with id_query.kind = id

pub mod query;

use self::query::{DataLocation, ElementDataQuery, ElementKind, ElementQuery};
use super::TransformEntry;
use crate::{
	action::transform::result::{TransformResult as TrRes, TransformedEntry, TransformedMessage},
	entry::Entry,
	error::transform::{HtmlError, InvalidUrlError, NothingToTransformError},
	sink::Media,
	utils::OptionExt,
};

//use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use either::Either;
use html5ever::rcdom::Handle as HtmlNode;
use soup::{NodeExt, QueryBuilderExt, Soup};
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
	/// Don't error out if the HTML page contains an empty body, e.g. if we are being rate-limited
	pub ignore_empty: bool,
}

impl TransformEntry for Html {
	type Error = HtmlError;

	fn transform_entry(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		tracing::debug!("Parsing HTML");

		let soup = Soup::new(
			entry
				.raw_contents
				.as_ref()
				.ok_or(NothingToTransformError)?
				.as_str(),
		);

		let body = soup
			.get_handle()
			.tag("body") // use body as the root html node
			.find()
			.unwrap_or_else(|| {
				tracing::debug!("HTML doesn't contain a body, using the root as the body");

				// or use the entire html if it doesn't exist for some reason (I don't think it should?)
				soup.get_handle()
			});

		if body.text().trim().is_empty() {
			tracing::warn!("HTML body is completely empty");

			return if self.ignore_empty {
				Ok(Vec::new())
			} else {
				Err(NothingToTransformError.into())
			};
		}

		let items = match self.itemq.as_ref() {
			Some(itemq) => Either::Left(find_chain(&body, itemq)?.into_iter()),
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
impl Html {
	fn extract_entry(&self, html: &HtmlNode) -> Result<TransformedEntry, HtmlError> {
		let title = self
			.titleq
			.as_ref()
			.try_and_then(|q| extract_data(html, q))?;

		let body = self.textq.as_ref().try_map(|q| extract_body(html, q))?;
		let id = self.idq.as_ref().try_and_then(|q| extract_data(html, q))?;
		let link = self.linkq.as_ref().try_and_then(|q| extract_url(html, q))?;
		let img = self.imgq.as_ref().try_and_then(|q| extract_url(html, q))?;

		Ok(TransformedEntry {
			id: TrRes::Old(id),
			raw_contents: TrRes::Old(body.clone()), // TODO: add an ability to choose if raw_contents should be kept from prev step
			msg: TransformedMessage {
				title: TrRes::Old(title),
				body: TrRes::Old(body),
				link: TrRes::Old(link),
				media: TrRes::Old(img.map(|url| vec![Media::Photo(url)])),
			},
		})
	}
}

/// Extract data from the provided HTML tags and join them
fn extract_data(
	html: &HtmlNode,
	data_query: &ElementDataQuery,
) -> Result<Option<String>, HtmlError> {
	let data = find_chain(html, &data_query.query)?
		.into_iter()
		.map(|hndl| match &data_query.data_location {
			DataLocation::Text => Some(hndl.text()),
			DataLocation::Attr(v) => hndl.get(v),
		})
		.collect::<Option<Vec<_>>>();

	let data = match data {
		Some(v) => v,
		None if data_query.optional => return Ok(None),
		None => {
			return Err(HtmlError::DataNotFoundInElement {
				data: data_query.data_location.clone(),
				element: data_query.query.clone(),
			});
		}
	};

	let s = data.join("\n\n"); // lifetime workaround
	let s = s.trim();

	if s.is_empty() {
		return if data_query.optional {
			Ok(None)
		} else {
			Err(HtmlError::ElementEmpty(data_query.query.clone()))
		};
	}

	let s = match &data_query.regex {
		Some(r) => r.replace(s).into_owned(),
		None => s.to_owned(),
	};

	Ok(Some(s))
}

fn extract_body(html: &HtmlNode, data_queries: &[ElementDataQuery]) -> Result<String, HtmlError> {
	Ok(data_queries
		.iter()
		.map(|query| extract_data(html, query))
		.collect::<Result<Vec<_>, _>>()?
		.into_iter()
		.flatten()
		.collect::<Vec<_>>()
		.join("\n\n"))
}

fn extract_url(html: &HtmlNode, query: &ElementDataQuery) -> Result<Option<Url>, HtmlError> {
	extract_data(html, query)?
		.try_map(|url| Url::try_from(url.as_str()).map_err(|e| InvalidUrlError(e, url).into()))
}

/// Find all elements matching the query in all the provided HTML parts
fn find_chain(html: &HtmlNode, elem_queries: &[ElementQuery]) -> Result<Vec<HtmlNode>, HtmlError> {
	if elem_queries.is_empty() {
		return Ok(vec![html.get_handle()]);
	}

	let mut html_nodes = vec![html.get_handle()];

	for (i, elem_query) in elem_queries.iter().enumerate() {
		html_nodes = html_nodes
			.into_iter()
			.flat_map(|html| find(html, elem_query))
			.collect();

		if html_nodes.is_empty() {
			return Err(HtmlError::ElementNotFound {
				num: i,
				elem_list: elem_queries.to_vec(),
			});
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
