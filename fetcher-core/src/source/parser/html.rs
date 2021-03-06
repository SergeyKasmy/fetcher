/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
// TODO: better handle invalid config values
// TODO: make sure read_filter_type not_present_in_read_list only works with id_query.kind = id

pub mod query;

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use html5ever::rcdom::Handle;
use soup::{NodeExt, QueryBuilderExt, Soup};
use url::Url;

use self::query::{
	DataLocation, IdQuery, IdQueryKind, ImageQuery, Query, QueryData, QueryKind, TextQuery,
	TitleQuery, UrlQuery,
};
use crate::entry::Entry;
use crate::error::source::parse::HtmlError;
use crate::sink::{Media, Message};

#[derive(Debug)]
pub struct Html {
	pub itemq: Vec<Query>,
	pub titleq: Option<TitleQuery>,
	pub textq: Vec<TextQuery>, // allow to find multiple paragraphs and join them together
	pub idq: Option<IdQuery>,
	pub linkq: Option<UrlQuery>,
	pub imgq: Option<ImageQuery>,
}

impl Html {
	#[tracing::instrument(skip_all)]
	pub fn parse(&self, entry: Entry) -> Result<Vec<Entry>, HtmlError> {
		tracing::debug!("Parsing HTML");

		let soup = Soup::new(entry.msg.body.as_str());
		let items = find_chain(&soup, &self.itemq);

		let entries = items
			.map(|item| -> Result<Entry, HtmlError> {
				let id = self
					.idq
					.as_ref()
					.map(|idq| extract_id(&item, idq))
					.transpose()?;
				let link = self
					.linkq
					.as_ref()
					.map(|linkq| extract_url(&item, linkq))
					.transpose()?;
				let title = self
					.titleq
					.as_ref()
					.and_then(|titleq| extract_title(&item, titleq));
				let body = extract_body(&item, &self.textq);
				let img = if let Some(imgq) = self.imgq.as_ref() {
					extract_img(&item, imgq)?
				} else {
					None
				};

				Ok(Entry {
					id,
					msg: Message {
						title,
						body,
						link,
						media: img.map(|url| vec![Media::Photo(url)]),
					},
				})
			})
			.collect::<Result<Vec<_>, HtmlError>>()?;

		tracing::debug!("Found {num} HTML articles total", num = entries.len());

		Ok(entries)
	}
}
fn extract_url(item: &impl QueryBuilderExt, urlq: &UrlQuery) -> Result<Url, HtmlError> {
	let mut link = extract_data(&mut find_chain(item, &urlq.inner.query), &urlq.inner)
		.ok_or(HtmlError::UrlNotFound)?
		.trim()
		.to_owned();

	if let Some(prepend) = &urlq.prepend {
		link.insert_str(0, prepend);
	}

	Ok(Url::try_from(link.as_str())?)
}

fn extract_id(item: &impl QueryBuilderExt, idq: &IdQuery) -> Result<String, HtmlError> {
	let id_str = extract_data(&mut find_chain(item, &idq.inner.query), &idq.inner)
		.ok_or(HtmlError::IdNotFound)?
		.trim()
		.to_owned();

	Ok(match &idq.kind {
		IdQueryKind::String => id_str,
		IdQueryKind::Date => {
			todo!()
			// ArticleId::Date(match parse_pretty_date(&id_str) {
			// 	Ok(d) => d,
			// 	Err(e) if matches!(e, Error::InvalidDateTimeFormat(_)) => {
			// 		return None
			// 	}
			// 	Err(e) => return Some(Err(e)),
			// })
		}
	})
}

fn extract_title(item: &impl QueryBuilderExt, titleq: &TitleQuery) -> Option<String> {
	extract_data(&mut find_chain(item, &titleq.0.query), &titleq.0)
}

fn extract_body(item: &impl QueryBuilderExt, textq: &[TextQuery]) -> String {
	textq
		.iter()
		.filter_map(|x| {
			extract_data(&mut find_chain(item, &x.inner.query), &x.inner).map(|s| {
				let mut s = s.trim().to_string();
				if let Some(prepend) = x.prepend.as_deref() {
					s.insert_str(0, prepend);
				}

				s
			})
		})
		.collect::<Vec<_>>()
		.join("\n\n")
}

fn extract_img(item: &impl QueryBuilderExt, imgq: &ImageQuery) -> Result<Option<Url>, HtmlError> {
	let mut img_url = match extract_data(
		&mut find_chain(item, &imgq.url.inner.query), // TODO: check iterator not empty
		&imgq.url.inner,                              // TODO: make less fugly
	) {
		Some(s) => s.trim().to_owned(),
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

	if let Some(prepend) = &imgq.url.prepend {
		img_url.insert_str(0, prepend);
	}

	Ok(Some(Url::try_from(img_url.as_str())?))
}

/// Find items matching the query in the provided HTML part
fn find<'a>(
	qb: &impl QueryBuilderExt,
	q: &'a QueryKind,
	ignore: Option<&'a [QueryKind]>,
) -> Box<dyn Iterator<Item = Handle> + 'a> {
	Box::new(
		match q {
			QueryKind::Tag(val) => qb.tag(val.as_str()).find_all(),
			QueryKind::Class(val) => qb.class(val.as_str()).find_all(),
			QueryKind::Attr { name, value } => qb.attr(name.as_str(), value.as_str()).find_all(),
		}
		.filter(move |found| {
			if let Some(ignore) = ignore {
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
			None => find(qb, &q.kind, q.ignore.as_deref()),
			Some(handles) => {
				Box::new(handles.flat_map(|hndl| find(&hndl, &q.kind, q.ignore.as_deref())))
			}
		});
	}

	handles.unwrap() // unwrap NOTE: safe *if* there are more than 0 query kinds which should be always... hopefully... // TODO: make sure there are more than 0 qks
}

/// Extract data from the provided HTML tags and join them
fn extract_data(h: &mut dyn Iterator<Item = Handle>, q: &QueryData) -> Option<String> {
	// debug_assert!(
	// 	h.peekable().peek().is_some(),
	// 	"No HTML tags to extract data from"
	// );

	let data = h
		.map(|hndl| match &q.data_location {
			DataLocation::Text => Some(hndl.text()),
			DataLocation::Attr(v) => hndl.get(v),
		})
		.collect::<Option<Vec<_>>>();

	data.map(|s| s.join("\n\n"))
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
	const TODAY_WORDS: &[&str] = &["Today", "Heute", "??????????????"];
	const YESTERDAY_WORDS: &[&str] = &["Yesterday", "Gestern", "??????????"];

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
