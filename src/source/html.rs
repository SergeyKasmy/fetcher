use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use html5ever::rcdom::Handle;
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};
use url::Url;

use crate::error::Result;
use crate::sink::Message;
use crate::source::Responce;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Amount {
	First,
	All,
}

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
	// amount: Amount,
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
	// TODO: make sure it's always Amount::First
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
}

impl Html {
	pub async fn get(&self) -> Result<Vec<Responce>> {
		let page = reqwest::get(self.url.as_str())
			.await
			.unwrap()
			.text()
			.await
			.unwrap();

		let soup = Soup::new(page.as_str());
		let items = Self::find_chain(&soup, &self.itemq);

		let responces = items
			.into_iter()
			// TODO: filter read items by id
			.map(|hndl| {
				let link = {
					let mut link = Self::extract_data(
						&mut Self::find_chain(&hndl, &self.linkq.inner.kind),
						&self.linkq.inner,
					);
					if let Some(s) = &self.linkq.prepend {
						link.insert_str(0, s);
					}
					link
				};

				let text = {
					let mut text = self
						.textq
						.iter()
						.map(|x| Self::extract_data(&mut Self::find_chain(&hndl, &x.kind), x))
						.collect::<Vec<_>>()
						.join("\n\n");

					text.push_str(&format!("\n\n<a href=\"{link}\">Link</a>"));

					text
				};

				let id = {
					let id_str = Self::extract_data(
						&mut Self::find_chain(&hndl, &self.idq.inner.kind),
						&self.idq.inner,
					);

					match &self.idq.kind {
						IdQueryKind::String => id_str,
						IdQueryKind::Date => Self::parse_pretty_date(&id_str).to_string(),
					}
				};

				Responce {
					id: Some(id),
					msg: Message { text, media: None },
				}
			})
			.inspect(|x| {
				dbg!(x);
			})
			.collect();

		Ok(responces)
	}

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

	fn find_chain<'a>(
		qb: &impl QueryBuilderExt,
		qs: &'a [QueryKind],
	) -> Box<dyn Iterator<Item = Handle> + 'a> {
		let mut handles: Option<Box<dyn Iterator<Item = Handle>>> = None;

		for q in qs {
			handles = Some(match handles {
				None => Self::find(qb, q),
				Some(handles) => Box::new(handles.map(|hndl| Self::find(&hndl, q)).flatten()),
			});
		}

		handles.unwrap()
	}

	fn extract_data(h: &mut dyn Iterator<Item = Handle>, q: &Query) -> String {
		let data = h
			.map(|x| match &q.data_location {
				DataLocation::Text => x.text(),
				DataLocation::Attr { value } => x.get(value).unwrap(),
			})
			.collect::<Vec<_>>();

		// match q.amount {
		// 	Amount::First => data.remove(0),
		// 	Amount::All => data.join("\n\n"),
		// }
		data.join("\n\n")
	}

	fn parse_pretty_date(mut date_str: &str) -> DateTime<Utc> {
		enum DateTimeKind {
			Today,
			Yesterday,
			Other,
		}

		// TODO: properly parse different languages
		const YESTERDAY_WORDS: &[&str] = &["Yesterday", "Gestern", "Вчера"];
		const TODAY_WORDS: &[&str] = &["Today", "Heute", "Сегодня"];

		date_str = date_str.trim();

		let mut datetime_kind = DateTimeKind::Other;
		for w in YESTERDAY_WORDS {
			if date_str.starts_with(w) {
				date_str = &date_str[w.len()..];
				datetime_kind = DateTimeKind::Yesterday;
			}
		}

		for w in TODAY_WORDS {
			if date_str.starts_with(w) {
				date_str = &date_str[w.len()..];
				datetime_kind = DateTimeKind::Today;
			}
		}

		if date_str.starts_with(',') {
			date_str = &date_str[1..];
		}

		date_str = date_str.trim();

		match datetime_kind {
			DateTimeKind::Yesterday => {
				let time = NaiveTime::parse_from_str(date_str, "%H:%M").unwrap();
				// date.date().pred().and_time(date.time()).unwrap(); // TODO: why does .and_time() return an option????
				Local::today().pred().and_time(time).unwrap().into()
			}
			DateTimeKind::Today => {
				let time = NaiveTime::parse_from_str(date_str, "%H:%M").unwrap();
				Local::today().and_time(time).unwrap().into()
			}
			DateTimeKind::Other => Utc
				.from_local_datetime(
					&NaiveDate::parse_from_str(date_str, "%d.%m.%Y")
						.unwrap()
						.and_hms(0, 0, 0),
				)
				.unwrap(),
		}
	}
}
