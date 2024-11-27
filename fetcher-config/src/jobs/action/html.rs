/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod query;

use self::query::{ElementDataQuery, ElementQuery, ItemQuery};
use crate::FetcherConfigError;
use fetcher_core::{action::transform::entry::html::Html as CHtml, utils::OptionExt};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Html {
	pub item: Option<ItemQuery>,
	pub title: Option<ElementDataQuery>,
	pub text: Option<Vec<ElementDataQuery>>,
	pub id: Option<ElementDataQuery>,
	pub link: Option<ElementDataQuery>,
	pub img: Option<ElementDataQuery>,
}

impl Html {
	pub fn decode_from_conf(self) -> Result<CHtml, FetcherConfigError> {
		Ok(CHtml {
			item: self.item.map(|x| {
				x.query
					.into_iter()
					.map(ElementQuery::decode_from_conf)
					.collect()
			}),
			title: self.title.try_map(ElementDataQuery::decode_from_conf)?,
			text: self.text.try_map(|v| {
				v.into_iter()
					.map(ElementDataQuery::decode_from_conf)
					.collect::<Result<_, _>>()
			})?,
			id: self.id.try_map(ElementDataQuery::decode_from_conf)?,
			link: self.link.try_map(ElementDataQuery::decode_from_conf)?,
			img: self.img.try_map(ElementDataQuery::decode_from_conf)?,
		})
	}
}
