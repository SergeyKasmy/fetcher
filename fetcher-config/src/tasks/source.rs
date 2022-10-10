/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod email;
pub mod file;
pub mod http;
pub mod reddit;
pub mod twitter;

use self::{email::Email, file::File, http::Http, reddit::Reddit, twitter::Twitter};
use crate::{tasks::external_data::ExternalData, Error};
use fetcher_core::{read_filter::ReadFilter as CReadFilter, source};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(clippy::large_enum_variant)]
#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum Source {
	WithSharedReadFilter(#[serde_as(deserialize_as = "OneOrMany<_>")] Vec<WithSharedReadFilter>),
	WithCustomReadFilter(WithCustomReadFilter),
}

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WithSharedReadFilter {
	Http(Http),
	Twitter(Twitter),
	File(File),
	Reddit(Reddit),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum WithCustomReadFilter {
	Email(Email),
}

// TODO: clean up
impl Source {
	pub fn parse(
		self,
		rf: Option<Arc<RwLock<CReadFilter>>>,
		external: &dyn ExternalData,
	) -> Result<source::Source, Error> {
		Ok(match self {
			Source::WithSharedReadFilter(v) => {
				let sources = v
					.into_iter()
					.map(|x| {
						Ok(match x {
							WithSharedReadFilter::Http(x) => {
								source::WithSharedRFKind::Http(x.parse()?)
							}
							WithSharedReadFilter::Twitter(x) => {
								source::WithSharedRFKind::Twitter(x.parse(external)?)
							}
							WithSharedReadFilter::File(x) => {
								source::WithSharedRFKind::File(x.parse())
							}
							WithSharedReadFilter::Reddit(x) => {
								source::WithSharedRFKind::Reddit(x.parse())
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
					source::WithCustomRF::Email(x.parse(external)?),
				),
			},
		})
	}
}
