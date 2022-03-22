/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

pub mod email;
pub mod html;
pub mod rss;
pub mod twitter;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::{read_filter, source};

use self::email::Email;
use self::html::Html;
use self::rss::Rss;
use self::twitter::Twitter;

use super::DataSettings;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum Source {
	// WithSharedReadFilter(OneOrMultiple<WithSharedReadFilter>),
	WithSharedReadFilter(Vec<WithSharedReadFilter>),
	WithCustomReadFilter(WithCustomReadFilter),
}

// #[derive(Deserialize, Serialize, Debug)]
// pub(crate) enum OneOrMultiple<T> {
// 	One(T),
// 	Multiple(Vec<T>),
// }

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WithSharedReadFilter {
	Html(Html),
	Rss(Rss),
	Twitter(Twitter),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WithCustomReadFilter {
	Email(Email),
}

impl Source {
	pub(crate) async fn parse(
		self,
		name: &str,
		settings: &DataSettings,
		default_read_filter_kind: Option<read_filter::Kind>,
	) -> Result<source::Source> {
		Ok(match self {
			Source::WithSharedReadFilter(v) => {
				let inner = v
					.into_iter()
					.map(|x| {
						Ok(match x {
							WithSharedReadFilter::Html(x) => {
								source::WithSharedReadFilterInner::Html(x.parse())
							}
							WithSharedReadFilter::Rss(x) => {
								source::WithSharedReadFilterInner::Rss(x.parse())
							}
							WithSharedReadFilter::Twitter(x) => {
								source::WithSharedReadFilterInner::Twitter(x.parse(settings)?)
							}
						})
					})
					.collect::<Result<Vec<_>>>()?;

				source::Source::WithSharedReadFilter(source::WithSharedReadFilter::new(
					inner,
					(settings.read_filter)(name.to_owned(), default_read_filter_kind)
						.await?
						.unwrap(), // unwrap FIXME: remove when settings::read_filter::get gets updated
				))
			}
			Source::WithCustomReadFilter(s) => match s {
				WithCustomReadFilter::Email(x) => source::Source::WithCustomReadFilter(
					source::WithCustomReadFilter::Email(x.parse(settings)?),
				),
			},
		})
	}
}
