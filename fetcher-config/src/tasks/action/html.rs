/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod query;

use self::query::{ElementDataQuery, ElementQuery};
use crate::Error;
use fetcher_core::{action::transform::Html as CoreHtml, utils::OptionExt};

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

	pub ignore_empty: Option<bool>,
}

impl Html {
	pub fn parse(self) -> Result<CoreHtml, Error> {
		Ok(CoreHtml {
			itemq: self
				.itemq
				.map(|v| v.into_iter().map(ElementQuery::parse).collect()),
			titleq: self.titleq.try_map(ElementDataQuery::parse)?,
			textq: self.textq.try_map(|v| {
				v.into_iter()
					.map(ElementDataQuery::parse)
					.collect::<Result<_, _>>()
			})?,
			idq: self.idq.try_map(ElementDataQuery::parse)?,
			linkq: self.linkq.try_map(ElementDataQuery::parse)?,
			imgq: self.imgq.try_map(ElementDataQuery::parse)?,
			ignore_empty: self.ignore_empty.unwrap_or(true),
		})
	}
}
