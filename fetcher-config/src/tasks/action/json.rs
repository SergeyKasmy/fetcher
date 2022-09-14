/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use fetcher_core::action::transform::entry::json as core_json;

#[derive(Deserialize, Serialize, Debug)]
pub struct TextQuery {
	pub string: String,
	pub prepend: Option<String>,
	pub append: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Json {
	#[serde(rename = "item_query")]
	pub itemq: Vec<String>,

	#[serde(rename = "title_query")]
	pub titleq: Option<String>,

	#[serde(rename = "text_query")]
	pub textq: Option<Vec<TextQuery>>,

	#[serde(rename = "id_query")]
	pub idq: String,

	#[serde(rename = "link_query")]
	pub linkq: Option<TextQuery>,

	#[serde(rename = "img_query")]
	pub imgq: Option<Vec<String>>,
}

impl TextQuery {
	pub fn parse(self) -> core_json::TextQuery {
		core_json::TextQuery {
			string: self.string,
			prepend: self.prepend,
			append: self.append,
		}
	}
}

impl Json {
	pub fn parse(self) -> core_json::Json {
		core_json::Json {
			itemq: self.itemq,
			titleq: self.titleq,
			textq: self
				.textq
				.map(|v| v.into_iter().map(TextQuery::parse).collect::<_>()),
			idq: self.idq,
			linkq: self.linkq.map(TextQuery::parse),
			imgq: self.imgq,
		}
	}
}
