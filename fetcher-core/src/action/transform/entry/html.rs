/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
// TODO: better handle invalid config values
// TODO: make sure read_filter_type not_present_in_read_list only works with id_query.kind = id

pub mod query;

use self::query::{DataLocation, ImageQuery, Query, QueryData, QueryKind};
use super::TransformEntry;
use crate::action::transform::result::{
	TransformResult as TrRes, TransformedEntry, TransformedMessage,
};
use crate::entry::Entry;
use crate::error::transform::HtmlError;
use crate::error::transform::InvalidUrlError;
use crate::error::transform::NothingToTransformError;
use crate::sink::Media;

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use html5ever::rcdom::Handle;
use soup::{NodeExt, QueryBuilderExt, Soup};
use url::Url;

#[derive(Debug)]
pub struct Html {
	// TODO: make optional
	pub itemq: Vec<Query>,
	pub titleq: Option<QueryData>,
	pub textq: Option<Vec<QueryData>>, // allow to find multiple paragraphs and join them together
	pub idq: Option<QueryData>,
	pub linkq: Option<QueryData>,
	pub imgq: Option<ImageQuery>,
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
		let items = find_chain(&soup, &self.itemq);

		let entries = items
			.map(|item| self.extract_entry(&item))
			.collect::<Result<Vec<_>, _>>()?;

		tracing::debug!("Found {num} HTML articles total", num = entries.len());

		Ok(entries)
	}
}

// TODO: make sure (and add tests!) that it errors if no item was found
impl Html {
	fn extract_entry(&self, item: &impl QueryBuilderExt) -> Result<TransformedEntry, HtmlError> {
		let id = self
			.idq
			.as_ref()
			.map(|idq| extract_data(item, idq)?.ok_or(HtmlError::IdNotFound))
			.transpose()?;

		let link = self
			.linkq
			.as_ref()
			.map(|linkq| extract_url(item, linkq))
			.transpose()?;

		let title = self
			.titleq
			.as_ref()
			.map(|titleq| extract_data(item, titleq)?.ok_or(HtmlError::TitleNotFound))
			.transpose()?;

		let body = match &self.textq {
			Some(textq) => Some(extract_body(item, textq)?),
			None => None,
		};

		let img = match &self.imgq {
			Some(imgq) => extract_img(item, imgq)?,
			None => None,
		};

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

fn extract_url(item: &impl QueryBuilderExt, urlq: &QueryData) -> Result<Url, HtmlError> {
	let link = extract_data(item, urlq)?.ok_or(HtmlError::UrlNotFound)?;

	Url::try_from(link.as_str()).map_err(|e| InvalidUrlError(e, link).into())
}

fn extract_body(item: &impl QueryBuilderExt, textq: &[QueryData]) -> Result<String, HtmlError> {
	Ok(textq
		.iter()
		.map(|query| extract_data(item, query))
		.collect::<Result<Vec<_>, _>>()?
		.into_iter()
		.flatten()
		.collect::<Vec<_>>()
		.join("\n\n"))
}

fn extract_img(item: &impl QueryBuilderExt, imgq: &ImageQuery) -> Result<Option<Url>, HtmlError> {
	let img_url = match extract_data(item, &imgq.inner)? {
		Some(s) => s,
		None => {
			if imgq.optional {
				tracing::trace!(
					"Found no image for the provided query but it's optional, skipping..."
				);
				return Ok(None);
			}

			return Err(HtmlError::ImageNotFound);
		}
	};

	Ok(Some(
		Url::try_from(img_url.as_str()).map_err(|e| InvalidUrlError(e, img_url))?,
	))
}

/// Extract data from the provided HTML tags and join them
fn extract_data(
	item: &impl QueryBuilderExt,
	query_data: &QueryData,
) -> Result<Option<String>, HtmlError> {
	let data = find_chain(item, &query_data.query)
		.map(|hndl| match &query_data.data_location {
			DataLocation::Text => Some(hndl.text()),
			DataLocation::Attr(v) => hndl.get(v),
		})
		.collect::<Option<Vec<_>>>();

	let data = match data {
		Some(v) => v,
		None => return Ok(None),
	};

	let s = data.join("\n\n"); // lifetime workaround
	let s = s.trim();

	if s.is_empty() {
		return Ok(None);
	}

	let s = match &query_data.regex {
		Some(r) => r.replace(s).into_owned(),
		None => s.to_owned(),
	};

	Ok(Some(s))
}

/// Find items matching the query in the provided HTML part
fn find<'a>(qb: &impl QueryBuilderExt, q: &'a Query) -> Box<dyn Iterator<Item = Handle> + 'a> {
	Box::new(
		match &q.kind {
			QueryKind::Tag(val) => qb.tag(val.as_str()).find_all(),
			QueryKind::Class(val) => qb.class(val.as_str()).find_all(),
			QueryKind::Attr { name, value } => qb.attr(name.as_str(), value.as_str()).find_all(),
		}
		.filter(move |found| {
			if let Some(ignore) = &q.ignore {
				for i in ignore.iter() {
					let should_be_ignored = match i {
						QueryKind::Tag(tag) => found.name() == tag,
						QueryKind::Class(class) => {
							found.get("class").map_or(false, |c| &c == class)
						}
						QueryKind::Attr { name, value } => {
							found.get(name).map_or(false, |a| &a == value)
						}
					};

					if should_be_ignored {
						return false;
					}
				}
			}

			true
		}),
	)
}

/// Find all items matching the query in all the provided HTML parts
fn find_chain<'a>(
	qb: &impl QueryBuilderExt,
	qs: &'a [Query],
) -> Box<dyn Iterator<Item = Handle> + 'a> {
	// debug_assert!(!qs.is_empty());
	let mut handles: Option<Box<dyn Iterator<Item = Handle>>> = None;

	for q in qs {
		handles = Some(match handles {
			None => find(qb, q),
			Some(handles) => Box::new(handles.flat_map(|hndl| find(&hndl, q))),
		});
	}

	handles.unwrap() // unwrap NOTE: safe *if* there are more than 0 query kinds which should be always... hopefully... // TODO: make sure there are more than 0 qks
}

// TODO: rewrite the entire fn to use today/Yesterday words and date formats from the config per source and not global
#[allow(dead_code)]
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
