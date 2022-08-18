/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub(crate) mod query;

use serde::{Deserialize, Serialize};

use self::query::{IdQuery, ImageQuery, Query, TextQuery, TitleQuery, UrlQuery};
use crate::error::ConfigError;
use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Html {
	#[serde(rename = "item_query")]
	pub(crate) itemq: Vec<Query>,

	#[serde(rename = "title_query")]
	pub(crate) titleq: Option<TitleQuery>,

	#[serde(rename = "text_query")]
	pub(crate) textq: Vec<TextQuery>,

	#[serde(rename = "id_query")]
	pub(crate) idq: Option<IdQuery>,

	#[serde(rename = "link_query")]
	pub(crate) linkq: Option<UrlQuery>,

	#[serde(rename = "img_query")]
	pub(crate) imgq: Option<ImageQuery>,
}

impl Html {
	pub(crate) fn parse(self) -> Result<source::parser::Html, ConfigError> {
		Ok(source::parser::Html {
			itemq: self.itemq.into_iter().map(Query::parse).collect(),
			titleq: self.titleq.map(TitleQuery::parse).transpose()?,
			textq: self
				.textq
				.into_iter()
				.map(TextQuery::parse)
				.collect::<Result<_, _>>()?,
			idq: self.idq.map(IdQuery::parse).transpose()?,
			linkq: self.linkq.map(UrlQuery::parse).transpose()?,
			imgq: self.imgq.map(ImageQuery::parse).transpose()?,
		})
	}
}
