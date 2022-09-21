/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod email;
pub mod file;
pub mod http;
pub mod twitter;

use self::email::Email;
use self::file::File;
use self::http::Http;
use self::twitter::Twitter;
use crate::Error;
use crate::{tasks::TaskSettings, OneOrMultiple};
use fetcher_core::read_filter::ReadFilter as CReadFilter;
use fetcher_core::source;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Source {
	WithSharedReadFilter(OneOrMultiple<WithSharedReadFilter>),
	WithCustomReadFilter(WithCustomReadFilter),
}

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WithSharedReadFilter {
	Http(Http),
	Twitter(Twitter),
	File(File),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WithCustomReadFilter {
	Email(Email),
}

impl Source {
	pub fn parse(
		self,
		rf: Option<Arc<RwLock<CReadFilter>>>,
		settings: &dyn TaskSettings,
	) -> Result<source::Source, Error> {
		Ok(match self {
			Source::WithSharedReadFilter(v) => {
				let v: Vec<WithSharedReadFilter> = v.into();

				let sources = v
					.into_iter()
					.map(|x| {
						Ok(match x {
							WithSharedReadFilter::Http(x) => {
								source::WithSharedRFKind::Http(x.parse()?)
							}
							WithSharedReadFilter::Twitter(x) => {
								source::WithSharedRFKind::Twitter(x.parse(settings)?)
							}
							WithSharedReadFilter::File(x) => {
								source::WithSharedRFKind::File(x.parse())
							}
						})
					})
					.collect::<Result<Vec<_>, Error>>()?;

				source::Source::WithSharedReadFilter {
					rf,
					kind: source::WithSharedRF::new(sources)
						.map_err(|e| Error::FetcherCoreSource(Box::new(e)))?,
				}
			}
			Source::WithCustomReadFilter(s) => match s {
				WithCustomReadFilter::Email(x) => source::Source::WithCustomReadFilter(
					source::WithCustomRF::Email(x.parse(settings)?),
				),
			},
		})
	}
}
