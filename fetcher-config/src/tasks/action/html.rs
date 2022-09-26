/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod query;

use self::query::{ElementDataQuery, ElementQuery};
use crate::Error;
use fetcher_core::action::transform::Html as CoreHtml;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Html {
	#[serde(rename = "item_query")]
	pub itemq: Option<Vec<ElementQuery>>,

	#[serde(rename = "title_query")]
	pub titleq: Option<ElementDataQuery>,

	#[serde(rename = "text_query")]
	pub textq: Option<Vec<ElementDataQuery>>,

	#[serde(rename = "id_query")]
	pub idq: Option<ElementDataQuery>,

	#[serde(rename = "link_query")]
	pub linkq: Option<ElementDataQuery>,

	#[serde(rename = "img_query")]
	pub imgq: Option<ElementDataQuery>,
}

impl Html {
	pub fn parse(self) -> Result<CoreHtml, Error> {
		Ok(CoreHtml {
			itemq: self
				.itemq
				.map(|v| v.into_iter().map(ElementQuery::parse).collect()),
			titleq: self.titleq.map(ElementDataQuery::parse).transpose()?,
			textq: self
				.textq
				.map(|v| {
					v.into_iter()
						.map(ElementDataQuery::parse)
						.collect::<Result<_, _>>()
				})
				.transpose()?,
			idq: self.idq.map(ElementDataQuery::parse).transpose()?,
			linkq: self.linkq.map(ElementDataQuery::parse).transpose()?,
			imgq: self.imgq.map(ElementDataQuery::parse).transpose()?,
		})
	}
}
