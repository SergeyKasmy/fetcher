pub(crate) mod query;

use serde::{Deserialize, Serialize};

use crate::source;

use self::query::{IdQuery, ImageQuery, LinkQuery, Query, TextQuery};

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Html {
	#[serde(rename = "item_query")]
	pub(crate) itemq: Vec<Query>,

	#[serde(rename = "text_query")]
	pub(crate) textq: Vec<TextQuery>,

	#[serde(rename = "id_query")]
	pub(crate) idq: IdQuery,

	#[serde(rename = "link_query")]
	pub(crate) linkq: LinkQuery,

	#[serde(rename = "img_query")]
	pub(crate) imgq: Option<ImageQuery>,
}

impl Html {
	pub(crate) fn parse(self) -> source::parser::Html {
		source::parser::Html {
			itemq: self.itemq.into_iter().map(Query::parse).collect(),
			textq: self.textq.into_iter().map(TextQuery::parse).collect(),
			idq: self.idq.parse(),
			linkq: self.linkq.parse(),
			imgq: self.imgq.map(ImageQuery::parse),
		}
	}
}
