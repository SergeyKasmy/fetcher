/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod query;

use self::query::{ImageQuery, Query, QueryData};
use crate::Error;
use fetcher_core::action::transform::Html as CoreHtml;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Html {
	#[serde(rename = "item_query")]
	pub itemq: Option<Vec<Query>>,

	#[serde(rename = "title_query")]
	pub titleq: Option<QueryData>,

	#[serde(rename = "text_query")]
	pub textq: Option<Vec<QueryData>>,

	#[serde(rename = "id_query")]
	pub idq: Option<QueryData>,

	#[serde(rename = "link_query")]
	pub linkq: Option<QueryData>,

	#[serde(rename = "img_query")]
	pub imgq: Option<ImageQuery>,
}

impl Html {
	pub fn parse(self) -> Result<CoreHtml, Error> {
		Ok(CoreHtml {
			itemq: self
				.itemq
				.map(|v| v.into_iter().map(Query::parse).collect()),
			titleq: self.titleq.map(QueryData::parse).transpose()?,
			textq: self
				.textq
				.map(|v| {
					v.into_iter()
						.map(QueryData::parse)
						.collect::<Result<_, _>>()
				})
				.transpose()?,
			idq: self.idq.map(QueryData::parse).transpose()?,
			linkq: self.linkq.map(QueryData::parse).transpose()?,
			imgq: self.imgq.map(ImageQuery::parse).transpose()?,
		})
	}
}
