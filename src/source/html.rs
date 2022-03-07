/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: better handle invalid config values
// TODO: make sure read_filter_type not_present_in_read_list only works with id_query.kind = id

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use html5ever::rcdom::Handle;
use soup::{NodeExt, QueryBuilderExt, Soup};
use url::Url;

use crate::error::{Error, Result};
use crate::read_filter::ReadFilter;
use crate::sink::message::{Link, LinkLocation};
use crate::sink::{Media, Message};
use crate::source::Responce;

#[derive(Clone, Debug)]
pub enum QueryKind {
	Tag { value: String },
	Class { value: String },
	Attr { name: String, value: String },
}

#[derive(Debug)]
pub enum DataLocation {
	Text,
	Attr { value: String },
}

#[derive(Debug)]
pub struct Query {
	pub(crate) kind: Vec<QueryKind>,
	pub(crate) data_location: DataLocation,
}

#[derive(Debug)]
pub struct TextQuery {
	pub(crate) prepend: Option<String>,
	pub(crate) inner: Query,
}

#[derive(Debug)]
pub enum IdQueryKind {
	String,
	Date,
}

#[derive(Debug)]
pub struct IdQuery {
	pub(crate) kind: IdQueryKind,
	pub(crate) inner: Query,
}

#[derive(Debug)]
pub struct LinkQuery {
	pub(crate) prepend: Option<String>,
	pub(crate) inner: Query,
}

#[derive(Debug)]
pub struct ImageQuery {
	pub(crate) optional: bool,
	pub(crate) inner: LinkQuery,
}

#[derive(Debug)]
pub struct Html {
	pub(crate) url: Url,
	pub(crate) itemq: Vec<QueryKind>,
	// TODO: make a separate title_query: Option<TextQuery> and allow to put a link into it
	pub(crate) textq: Vec<TextQuery>,
	pub(crate) idq: IdQuery,
	pub(crate) linkq: LinkQuery,
	pub(crate) imgq: Option<ImageQuery>,
}

impl Html {
	#[tracing::instrument(skip_all)]
	pub async fn get(&self, read_filter: &ReadFilter) -> Result<Vec<Responce>> {
		tracing::debug!("Fetching HTML source");
		let page = reqwest::get(self.url.as_str()).await?.text().await?;

		let soup = Soup::new(page.as_str());
		let items = Self::find_chain(&soup, &self.itemq);

		// TODO: mb move to source and make it generic for every source?
		#[derive(Debug)]
		enum Id {
			String(String),
			Date(DateTime<Utc>),
		}

		#[derive(Debug)]
		struct Article {
			id: Id,
			body: String,
			link: Url,
			img: Option<Url>,
		}

		let mut articles = items
			.filter_map(|item| {
				let link: Url = {
					let mut link = match Self::extract_data(
						&mut Self::find_chain(&item, &self.linkq.inner.kind),
						&self.linkq.inner,
					)
					.ok_or(Error::Html("link not found"))
					{
						Ok(s) => s.trim().to_string(),
						Err(e) => return Some(Err(e)),
					};

					if let Some(prepend) = &self.linkq.prepend {
						link.insert_str(0, prepend);
					}

					link.as_str().try_into().unwrap() // unwrap FIXME: pretty print if the found field isn't a valid url
				};

				let id = {
					let id_str = match Self::extract_data(
						&mut Self::find_chain(&item, &self.idq.inner.kind),
						&self.idq.inner,
					)
					.ok_or(Error::Html("id not found"))
					{
						Ok(s) => s.trim().to_string(),
						Err(e) => return Some(Err(e)),
					};

					match &self.idq.kind {
						IdQueryKind::String => Id::String(id_str),
						IdQueryKind::Date => Id::Date(match Self::parse_pretty_date(&id_str) {
							Ok(d) => d,
							Err(e) if matches!(e, Error::InvalidDateTimeFormat(_)) => return None,
							Err(e) => return Some(Err(e)),
						}),
					}
				};

				let body = self
					.textq
					.iter()
					.filter_map(|x| {
						Self::extract_data(&mut Self::find_chain(&item, &x.inner.kind), &x.inner)
							.map(|s| {
								let mut s = s.trim().to_string();
								if let Some(prepend) = x.prepend.as_deref() {
									s.insert_str(0, prepend);
								}

								s
							})
					})
					.collect::<Vec<_>>()
					.join("\n\n");

				let img: Option<Url> = match self.imgq.as_ref() {
					Some(img_query) => {
						let mut img_url = match Self::extract_data(
							&mut Self::find_chain(&item, &img_query.inner.inner.kind), // TODO: check iterator not empty
							&img_query.inner.inner,                                    // TODO: make less fugly
						) {
							Some(s) => s.trim().to_string(),
							None => {
								if img_query.optional {
									tracing::debug!("Found no image for the provided query but it's optional, skipping...");
									return None;
								} else {
									return Some(Err(Error::Html(
										"image not found but it's not optional",
									)));
								}
							}
						};

						if let Some(s) = &img_query.inner.prepend {
							img_url.insert_str(0, s);
						}

						match img_url
							.as_str()
							.try_into()
							.map_err(|_| Error::Html("The found url isn't an actual url!"))	// FiXME
						{
							Ok(v) => Some(v),
							Err(e) => return Some(Err(e)),
						}
					}
					None => None,
				};
				Some(Ok(Article {
					id,
					body,
					link,
					img,
				}))
			})
			.collect::<Result<Vec<_>>>()?;

		tracing::debug!("Found {num} HTML articles total", num = articles.len());

		if let Some(last_read_id) = read_filter.last_read() {
			if let Some(pos) = articles.iter().position(|x| match &x.id {
				Id::String(s) => s == last_read_id,
				Id::Date(d) => d <= &last_read_id.parse::<DateTime<Utc>>().unwrap(), // unwrap NOTE: should be safe, we parse in the same format we save
				                                                                     // TODO: add last_read_id format error for a nicer output
			}) {
				tracing::debug!(
					"Removing {num} already read HTML articles",
					num = articles.len() - pos
				);
				articles.drain(pos..);
			}
		}

		tracing::debug!("{num} unread HTML articles remaining", num = articles.len());

		Ok(articles
			.into_iter()
			.rev()
			.map(|a| Responce {
				id: Some(match a.id {
					Id::String(s) => s,
					Id::Date(d) => d.to_string(),
				}),
				msg: Message {
					title: None,
					body: a.body,
					link: Some(Link {
						url: a.link,
						loc: LinkLocation::Bottom,
					}),
					media: a.img.map(|u| vec![Media::Photo(u)]),
				},
			})
			.collect())
	}

	/// Find items matching the query in the provided HTML part
	fn find<'a>(
		qb: &impl QueryBuilderExt,
		q: &'a QueryKind,
	) -> Box<dyn Iterator<Item = Handle> + 'a> {
		match q {
			QueryKind::Tag { value } => qb.tag(value.as_str()).find_all(),
			QueryKind::Class { value } => qb.class(value.as_str()).find_all(),
			QueryKind::Attr { name, value } => qb.attr(name.as_str(), value.as_str()).find_all(),
		}
	}

	/// Find all items matching the query in all the provided HTML parts
	fn find_chain<'a>(
		qb: &impl QueryBuilderExt,
		qs: &'a [QueryKind],
	) -> Box<dyn Iterator<Item = Handle> + 'a> {
		// debug_assert!(!qs.is_empty());
		let mut handles: Option<Box<dyn Iterator<Item = Handle>>> = None;

		for q in qs {
			handles = Some(match handles {
				None => Self::find(qb, q),
				Some(handles) => Box::new(handles.flat_map(|hndl| Self::find(&hndl, q))),
			});
		}

		handles.unwrap() // unwrap NOTE: safe *if* there are more than 0 query kinds which should be always... hopefully... // TODO: make sure there are more than 0 qks
	}

	/// Extract data from the provided HTML tags and join them
	fn extract_data(h: &mut dyn Iterator<Item = Handle>, q: &Query) -> Option<String> {
		// debug_assert!(
		// 	h.peekable().peek().is_some(),
		// 	"No HTML tags to extract data from"
		// );

		let data = h
			.map(|hndl| match &q.data_location {
				DataLocation::Text => Some(hndl.text()),
				DataLocation::Attr { value } => hndl.get(value),
			})
			.collect::<Option<Vec<_>>>();

		data.map(|s| s.join("\n\n"))
	}

	// TODO: rewrite the entire fn to use today/Yesterday words and date formats from the config per source and not global
	fn parse_pretty_date(mut date_str: &str) -> Result<DateTime<Utc>> {
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

		date_str = date_str.trim_matches(',');
		date_str = date_str.trim();

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
				.from_local_datetime(
					&NaiveDate::parse_from_str(date_str, "%d.%m.%Y")?.and_hms(0, 0, 0),
				)
				.unwrap(), // unwrap NOTE: same as above
		})
	}
}
