use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Utc};
use html5ever::rcdom::Handle;
use serde::Deserialize;
use soup::{NodeExt, QueryBuilderExt, Soup};
use url::Url;

use crate::error::Result;
use crate::sink::Message;
use crate::source::Responce;

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(rename_all = "snake_case")]
// pub enum Amount {
// 	First,
// 	All,
// }

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
	pub async fn get(&self, last_read_id: Option<String>) -> Result<Vec<Responce>> {
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
		}

		let mut articles = items
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
						IdQueryKind::String => Id::String(id_str),
						IdQueryKind::Date => Id::Date(Self::parse_pretty_date(&id_str)?),
					}
				};

				Ok(Article { id, text })
			})
			.collect::<Result<Vec<_>>>()?;

		if let Some(last_read_id) = last_read_id {
			if let Some(pos) = articles.iter().position(|x| match &x.id {
				Id::String(s) => s == &last_read_id,
				Id::Date(d) => d <= &last_read_id.parse::<DateTime<Utc>>().unwrap(), // unwrap NOTE: should be safe, we parse in the same format we save
				                                                                     // TODO: add last_read_id format error for a nicer output
			}) {
				articles.drain(pos..);
			}
		}

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
					media: None,
				},
			})
			.collect())
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

		handles.unwrap() // unwrap NOTE: safe *if* there are more than 0 query kinds which should be always... hopefully... // TODO: make sure there are more than 0 qks
	}

	fn extract_data(h: &mut dyn Iterator<Item = Handle>, q: &Query) -> String {
		let data = h
			.map(|hndl| match &q.data_location {
				DataLocation::Text => hndl.text(),
				DataLocation::Attr { value } => hndl.get(value).expect("attr doesnt exist"), // FIXME
			})
			.collect::<Vec<_>>();

		// match q.amount {
		// 	Amount::First => data.remove(0),
		// 	Amount::All => data.join("\n\n"),
		// }
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
					&NaiveDate::parse_from_str(date_str, "%d.%m.%Y")
						.expect("HTML Date not in dd.mm.yyyy format") // FIXME
						.and_hms(0, 0, 0),
				)
				.unwrap(), // unwrap NOTE: same as above
		})
	}
}
