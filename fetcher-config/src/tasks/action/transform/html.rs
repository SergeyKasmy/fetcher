/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod query;

use self::query::{IdQuery, ImageQuery, Query, TextQuery, TitleQuery, UrlQuery};
use crate::error::ConfigError;
use fetcher_core::action::transform::Html as CoreHtml;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Html {
	#[serde(rename = "item_query")]
	pub itemq: Vec<Query>,

	#[serde(rename = "title_query")]
	pub titleq: Option<TitleQuery>,

	#[serde(rename = "text_query")]
	pub textq: Option<Vec<TextQuery>>,

	#[serde(rename = "id_query")]
	pub idq: Option<IdQuery>,

	#[serde(rename = "link_query")]
	pub linkq: Option<UrlQuery>,

	#[serde(rename = "img_query")]
	pub imgq: Option<ImageQuery>,
}

impl Html {
	pub fn parse(self) -> Result<CoreHtml, ConfigError> {
		Ok(CoreHtml {
			itemq: self.itemq.into_iter().map(Query::parse).collect(),
			titleq: self.titleq.map(TitleQuery::parse).transpose()?,
			textq: self
				.textq
				.map(|v| {
					v.into_iter()
						.map(TextQuery::parse)
						.collect::<Result<_, _>>()
				})
				.transpose()?,
			idq: self.idq.map(IdQuery::parse).transpose()?,
			linkq: self.linkq.map(UrlQuery::parse).transpose()?,
			imgq: self.imgq.map(ImageQuery::parse).transpose()?,
		})
	}
}
