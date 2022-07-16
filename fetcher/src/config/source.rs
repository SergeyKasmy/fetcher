/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod email;
pub mod file;
pub mod http;
pub mod parser;
pub mod twitter;

use serde::{Deserialize, Serialize};

use self::email::Email;
use self::file::File;
use self::http::Http;
use self::twitter::Twitter;
use super::{DataSettings, OneOrMultiple};
use crate::error::ConfigError;
use fetcher_core::{read_filter, source};

#[allow(clippy::large_enum_variant)] // don't care, it's used just once per task and isn't passed a lot
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum Source {
	WithSharedReadFilter(OneOrMultiple<WithSharedReadFilter>),
	WithCustomReadFilter(WithCustomReadFilter),
}

#[allow(clippy::large_enum_variant)] // don't care, it's used just once per task and isn't passed a lot
#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub(crate) enum WithSharedReadFilter {
	Http(Http),
	Twitter(Twitter),
	File(File),
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)// TODO: check if deny_unknown_fields can be used here, esp with flatten]
#[serde(rename_all = "snake_case")]
pub(crate) enum WithCustomReadFilter {
	Email(Email),
}

impl Source {
	pub(crate) async fn parse(
		self,
		name: &str,
		settings: &DataSettings,
		default_read_filter_kind: Option<read_filter::Kind>,
	) -> Result<source::Source, ConfigError> {
		Ok(match self {
			Source::WithSharedReadFilter(v) => {
				let v: Vec<WithSharedReadFilter> = v.into();

				let inner = v
					.into_iter()
					.map(|x| {
						Ok(match x {
							WithSharedReadFilter::Http(x) => {
								source::WithSharedReadFilterInner::Http(x.parse()?)
							}
							WithSharedReadFilter::Twitter(x) => {
								source::WithSharedReadFilterInner::Twitter(x.parse(settings)?)
							}
							WithSharedReadFilter::File(x) => {
								source::WithSharedReadFilterInner::File(x.parse())
							}
						})
					})
					.collect::<Result<Vec<_>, ConfigError>>()?;

				let read_filter =
					(settings.read_filter)(name.to_owned(), default_read_filter_kind).await?;

				source::Source::WithSharedReadFilter(
					source::WithSharedReadFilter::new(inner, read_filter)
						.map_err(|e| ConfigError::FetcherCoreSource(e.into()))?,
				)
			}
			Source::WithCustomReadFilter(s) => match s {
				WithCustomReadFilter::Email(x) => source::Source::WithCustomReadFilter(
					source::WithCustomReadFilter::Email(x.parse(settings)?),
				),
			},
		})
	}
}
