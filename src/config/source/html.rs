/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub(crate) mod query;

use serde::Deserialize;
use url::Url;

use crate::source;

use self::query::{IdQuery, ImageQuery, LinkQuery, Query, TextQuery};

#[derive(Deserialize, Debug)]
pub(crate) struct Html {
	pub(crate) url: Url,
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
	pub(crate) fn parse(self) -> source::Html {
		source::Html {
			url: self.url,
			itemq: self.itemq.into_iter().map(Query::parse).collect(),
			textq: self.textq.into_iter().map(TextQuery::parse).collect(),
			idq: self.idq.parse(),
			linkq: self.linkq.parse(),
			imgq: self.imgq.map(ImageQuery::parse),
		}
	}
}
