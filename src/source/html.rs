/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */
// TODO: better handle invalid config values
// TODO: make sure read_filter_type not_present_in_read_list only works with id_query.kind = id

pub(crate) mod query;

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use html5ever::rcdom::Handle;
use soup::{NodeExt, QueryBuilderExt, Soup};
use std::borrow::Cow;
use url::Url;

use self::query::{
	DataLocation, IdQuery, IdQueryKind, ImageQuery, LinkQuery, Query, QueryData, QueryKind,
	TextQuery,
};
use crate::error::{Error, Result};
use crate::read_filter::{Id, ReadFilter};
use crate::sink::message::{Link, LinkLocation};
use crate::sink::{Media, Message};
use crate::source::Responce;

#[derive(Debug)]
pub struct Html {
	pub(crate) url: Url,
	pub(crate) itemq: Vec<Query>,
	// TODO: make a separate title_query: Option<TextQuery> and allow to put a link into it
	pub(crate) textq: Vec<TextQuery>, // allow to find multiple paragraphs and join them together
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

		let mut articles = items
			.filter_map(|item| {
				let link: Url = {
					let mut link = match Self::extract_data(
						&mut Self::find_chain(&item, &self.linkq.inner.query),
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
						&mut Self::find_chain(&item, &self.idq.inner.query),
						&self.idq.inner,
					)
					.ok_or(Error::Html("id not found"))
					{
						Ok(s) => s.trim().to_string(),
						Err(e) => return Some(Err(e)),
					};

					match &self.idq.kind {
						IdQueryKind::String => ArticleId::String(id_str),
						IdQueryKind::Date => {
							ArticleId::Date(match Self::parse_pretty_date(&id_str) {
								Ok(d) => d,
								Err(e) if matches!(e, Error::InvalidDateTimeFormat(_)) => {
									return None
								}
								Err(e) => return Some(Err(e)),
							})
						}
					}
				};

				let body = self
					.textq
					.iter()
					.filter_map(|x| {
						Self::extract_data(&mut Self::find_chain(&item, &x.inner.query), &x.inner)
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

				let img = self
					.imgq
					.as_ref()
					.and_then(|img_query| {
						let mut img_url = match Self::extract_data(
							&mut Self::find_chain(&item, &img_query.inner.inner.query), // TODO: check iterator not empty
							&img_query.inner.inner,                                     // TODO: make less fugly
						) {
							Some(s) => s.trim().to_owned(),
							None => {
								if img_query.optional {
									tracing::debug!("Found no image for the provided query but it's optional, skipping...");
									return None;
								}

								return Some(Err(Error::Html(
									"image not found but it's not optional",
								)));
							}
						};

						if let Some(prepend) = &img_query.inner.prepend {
							img_url.insert_str(0, prepend);
						}

						Some(
							Url::try_from(img_url.as_str())
								.map_err(|_| Error::Html("The found url isn't an actual url!")),
						)
					})
					.transpose();

				let img = match img {
					Ok(v) => v,
					Err(e) => return Some(Err(e)),
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
		read_filter.remove_read_from(&mut articles);

		let unread_num = articles.len();
		if unread_num > 0 {
			tracing::info!("Found {unread_num} unread HTML articles");
		} else {
			tracing::debug!("All articles have already been read, none remaining to send");
		}

		Ok(articles
			.into_iter()
			.rev()
			.map(|a| Responce {
				id: Some(match a.id {
					ArticleId::String(s) => s,
					ArticleId::Date(d) => d.to_string(),
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
		ignore: &'a [QueryKind],
	) -> Box<dyn Iterator<Item = Handle> + 'a> {
		Box::new(
			match q {
				QueryKind::Tag { value } => qb.tag(value.as_str()).find_all(),
				QueryKind::Class { value } => qb.class(value.as_str()).find_all(),
				QueryKind::Attr { name, value } => {
					qb.attr(name.as_str(), value.as_str()).find_all()
				}
			}
			.filter(|found| {
				for i in ignore.iter() {
					let should_be_ignored = match i {
						QueryKind::Tag { value: tag } => found.name() == tag,
						QueryKind::Class { value: class } => {
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
				None => Self::find(qb, &q.kind, &q.ignore),
				Some(handles) => {
					Box::new(handles.flat_map(|hndl| Self::find(&hndl, &q.kind, &q.ignore)))
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

// TODO: mb move to source and make it generic for every source?
#[derive(Debug)]
enum ArticleId {
	String(String),
	Date(DateTime<Utc>),
}

#[derive(Debug)]
struct Article {
	id: ArticleId,
	body: String,
	link: Url,
	img: Option<Url>,
}

impl Id for Article {
	fn id(&self) -> Cow<'_, str> {
		match &self.id {
			ArticleId::String(s) => Cow::Borrowed(s.as_str()),
			ArticleId::Date(d) => todo!(),
		}
	}
}

/*
/// Checks if current read filter is compatible with current id query kind
fn read_filter_compatible(filter: &ReadFilter, idq_kind: IdQueryKind) -> bool {
	match filter.to_kind() {
		ReadFilterKind::NewerThanLastRead => true,
		ReadFilterKind::NotPresentInReadList => matches!(idq_kind, IdQueryKind::String),
	}
}
*/
