use html5ever::rcdom::Handle;
use serde::Deserialize;
use serde::Serialize;
use soup::NodeExt;
use soup::QueryBuilder;
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

// #[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case")]
// pub enum DataLocation {
// 	Text,
// 	Attr { value: String },
// }

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueryKind {
	Tag { value: String },
	Class { value: String },
	Attr { name: String, value: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Query {
	amount: Amount,
	kind: Vec<QueryKind>,
	// data_location: DataLocation,
}

#[derive(Deserialize, Debug)]
// TODO: use #[serde(try_from)]
pub struct Html {
	url: Url,
	item_query: Vec<QueryKind>,
	text_query: Query,
	id_query: Query,
}

impl Html {
	pub async fn get(&self) -> Result<Vec<Responce>> {
		let page = reqwest::get(self.url.as_str())
			.await
			.unwrap()
			.text()
			.await
			.unwrap();
		// let page = include_str!("../../debug_data/page.html");

		let soup = Soup::new(page.as_str());

		let items = Self::find_chain(&soup, &self.item_query);

		let responces = items
			.into_iter()
			.map(|x| {
				// let id = Self::find_chain(&x, &self.id_query)[0].text();

				let mut text = Self::find_chain(&x, &self.text_query.kind)
					.into_iter()
					.map(|x| x.text())
					.collect::<Vec<_>>();

				let text = match &self.text_query.amount {
					Amount::First => text.remove(0),
					Amount::All => text.join("\n\n"),
				};

				Responce {
					id: None,
					msg: Message { text, media: None },
				}
			})
			.collect();

		Ok(responces)
	}

	fn find(qb: &impl QueryBuilderExt, q: &QueryKind) -> Vec<Handle> {
		match q {
			QueryKind::Tag { value } => qb.tag(value.as_str()).find_all().collect(),
			QueryKind::Class { value } => qb.class(value.as_str()).find_all().collect(),
			QueryKind::Attr { name, value } => {
				qb.attr(name.as_str(), value.as_str()).find_all().collect()
			}
		}
	}

	fn find_chain(qb: &impl QueryBuilderExt, qs: &[QueryKind]) -> Vec<Handle> {
		let mut handles: Option<Vec<Handle>> = None;

		for q in qs {
			handles = Some(match &handles {
				None => Self::find(qb, q),
				Some(handles) => handles.iter().map(|x| Self::find(x, q)).flatten().collect(),
			});
		}

		handles.unwrap()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ser() {
		let q = Query {
			amount: Amount::All,
			kind: vec![
				QueryKind::Tag {
					value: "TagText".to_owned(),
				},
				QueryKind::Class {
					value: "ClassText".to_owned(),
				},
			],
		};

		println!("{}", toml::to_string_pretty(&q).unwrap());
	}
}
