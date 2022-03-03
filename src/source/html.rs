/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: better handle invalid config values

use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use html5ever::rcdom::Handle;
use serde::Deserialize;
use soup::{NodeExt, QueryBuilderExt, Soup};
use url::Url;

use crate::error::{Error, Result};
use crate::sink::{Media, Message};
use crate::source::Responce;

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryKind {
	Tag { value: String },
	Class { value: String },
	Attr { name: String, value: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DataLocation {
	Text,
	Attr { value: String },
}

#[derive(Deserialize, Debug)]
pub struct Query {
	kind: Vec<QueryKind>,
	data_location: DataLocation,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum IdQueryKind {
	String,
	Date,
}

#[derive(Deserialize, Debug)]
pub struct IdQuery {
	kind: IdQueryKind,
	#[serde(rename = "query")]
	inner: Query,
}

#[derive(Deserialize, Debug)]
pub struct LinkQuery {
	prepend: Option<String>,
	#[serde(flatten)]
	inner: Query,
}

#[derive(Deserialize, Debug)]
// TODO: use #[serde(try_from)]
pub struct Html {
	url: Url,
	#[serde(alias = "item_query")]
	itemq: Vec<QueryKind>,
	#[serde(alias = "text_query")]
	textq: Vec<Query>,
	#[serde(alias = "id_query")]
	idq: IdQuery,
	#[serde(alias = "link_query")]
	linkq: LinkQuery,
	#[serde(alias = "img_query")]
	imgq: Option<LinkQuery>,
}

impl Html {
	#[tracing::instrument(name = "Html::get", skip(self), fields(url = self.url.as_str()))]
	pub async fn get(&self, last_read_id: Option<String>) -> Result<Vec<Responce>> {
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
			text: String,
			img: Option<Url>,
		}

		let mut articles = items
			.filter_map(|item| {
				let link = {
					let mut link = Self::extract_data(
						&mut Self::find_chain(&item, &self.linkq.inner.kind),
						&self.linkq.inner,
					);
					if let Some(s) = &self.linkq.prepend {
						link.insert_str(0, s);
					}
					link
				};

				let id = {
					let id_str = Self::extract_data(
						&mut Self::find_chain(&item, &self.idq.inner.kind),
						&self.idq.inner,
					);

					match &self.idq.kind {
						IdQueryKind::String => Id::String(id_str),
						IdQueryKind::Date => Id::Date(match Self::parse_pretty_date(&id_str) {
							Ok(d) => d,
							Err(e) if matches!(e, Error::InvalidDateTimeFormat(_)) => return None,
							Err(e) => return Some(Err(e)),
						}),
					}
				};

				let text = {
					let mut text = self
						.textq
						.iter()
						.map(|x| Self::extract_data(&mut Self::find_chain(&item, &x.kind), x))
						.collect::<Vec<_>>()
						.join("\n\n");

					text.push_str(&format!("\n\n<a href=\"{link}\">Link</a>"));

					text
				};

				let img: Option<Url> = self.imgq.as_ref().map(|img_query| {
					let mut img_url = Self::extract_data(
						&mut Self::find_chain(&item, &img_query.inner.kind), // TODO: check iterator not empty
						&img_query.inner,
					);

					if let Some(s) = &img_query.prepend {
						img_url.insert_str(0, s);
					}

					img_url.as_str().try_into().unwrap()
				});
				Some(Ok(Article { id, text, img }))
			})
			.collect::<Result<Vec<_>>>()?;

		tracing::debug!("Found {num} HTML articles total", num = articles.len());

		if let Some(last_read_id) = last_read_id {
			if let Some(pos) = articles.iter().position(|x| match &x.id {
				Id::String(s) => s == &last_read_id,
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
					text: a.text,
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
	fn extract_data(h: &mut dyn Iterator<Item = Handle>, q: &Query) -> String {
		// debug_assert!(
		// 	h.peekable().peek().is_some(),
		// 	"No HTML tags to extract data from"
		// );

		let data = h
			.map(|hndl| match &q.data_location {
				DataLocation::Text => hndl.text(),
				DataLocation::Attr { value } => hndl.get(value).expect("attr doesnt exist"), // FIXME
			})
			.collect::<Vec<_>>();

		data.join("\n\n")
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
