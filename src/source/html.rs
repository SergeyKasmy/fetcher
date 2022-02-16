use html5ever::rcdom::Handle;
use serde::Deserialize;
use serde::Serialize;
use soup::NodeExt;
use soup::QueryBuilderExt;
use soup::Soup;
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
	amount: Amount,
	kind: Vec<QueryKind>,
	data_location: DataLocation,
}

#[derive(Deserialize, Debug)]
pub struct LinkQuery {
	// TODO: make sure it's always Amount::First
	prepend: Option<String>,
	#[serde(flatten)]
	query: Query,
}

#[derive(Deserialize, Debug)]
// TODO: use #[serde(try_from)]
pub struct Html {
	url: Url,
	item_query: Vec<QueryKind>,
	text_query: Vec<Query>,
	id_query: Query,
	link_query: LinkQuery,
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
		let items = Self::find_chain(&soup, &self.item_query);

		let responces = items
			.into_iter()
			.map(|hndl| {
				let link = {
					let mut link = Self::extract_data(
						&mut Self::find_chain(&hndl, &self.link_query.query.kind),
						&self.link_query.query,
					);
					if let Some(s) = &self.link_query.prepend {
						link.insert_str(0, s);
					}

					link
				};

				let text = {
					let mut text = self
						.text_query
						.iter()
						.map(|x| Self::extract_data(&mut Self::find_chain(&hndl, &x.kind), x))
						.collect::<Vec<_>>()
						.join("\n\n");

					text.push_str(&format!("\n\n<a href=\"{link}\">Link</a>"));

					text
				};

				let id = Self::extract_data(
					&mut Self::find_chain(&hndl, &self.id_query.kind),
					&self.id_query,
				);

				Responce {
					id: Some(id),
					msg: Message { text, media: None },
				}
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
		let mut data = h
			.map(|x| match &q.data_location {
				DataLocation::Text => x.text(),
				DataLocation::Attr { value } => x.get(value).unwrap(),
			})
			.collect::<Vec<_>>();

		match q.amount {
			Amount::First => data.remove(0),
			Amount::All => data.join("\n\n"),
		}
	}
}
