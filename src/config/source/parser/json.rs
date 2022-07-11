/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};

use crate::source;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TextQuery {
	pub(crate) string: String,
	pub(crate) prepend: Option<String>,
	pub(crate) append: Option<String>,
}

impl TextQuery {
	pub(crate) fn parse(self) -> source::parser::json::TextQuery {
		source::parser::json::TextQuery {
			string: self.string,
			prepend: self.prepend,
			append: self.append,
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Json {
	#[serde(rename = "item_query")]
	pub(crate) itemq: Vec<String>,

	#[serde(rename = "title_query")]
	pub(crate) titleq: Option<String>,

	#[serde(rename = "text_query")]
	pub(crate) textq: Vec<TextQuery>,

	#[serde(rename = "id_query")]
	pub(crate) idq: String,

	#[serde(rename = "link_query")]
	pub(crate) linkq: Option<TextQuery>,

	#[serde(rename = "img_query")]
	pub(crate) imgq: Option<Vec<String>>,
}

impl Json {
	pub(crate) fn parse(self) -> source::parser::Json {
		source::parser::Json {
			itemq: self.itemq,
			titleq: self.titleq,
			textq: self.textq.into_iter().map(TextQuery::parse).collect(),
			idq: self.idq,
			linkq: self.linkq.map(TextQuery::parse),
			imgq: self.imgq,
		}
	}
}
