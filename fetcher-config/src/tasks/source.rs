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
use crate::{tasks::external_data::ProvideExternalData, Error};
use fetcher_core::{read_filter::ReadFilter as CReadFilter, source::Source as CSource};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
	// with shared read filter
	Http(Http),
	Twitter(Twitter),
	File(File),
	Reddit(Reddit),

	// with custom read filter
	Email(Email),
}

impl Source {
	pub fn parse(
		self,
		rf: Option<Arc<RwLock<CReadFilter>>>,
		external: &dyn ProvideExternalData,
	) -> Result<CSource, Error> {
		// make a CSource::WithSharedReadFilter out of a CWithSharedRFKind
		macro_rules! WithSharedRF {
			($source:expr) => {
				CSource::WithSharedReadFilter { rf, kind: $source }
			};
		}

		Ok(match self {
			Self::Http(x) => WithSharedRF!(x.parse()?),
			Self::Twitter(x) => WithSharedRF!(x.parse(external)?),
			Self::File(x) => WithSharedRF!(x.parse()),
			Self::Reddit(x) => WithSharedRF!(x.parse()),
			Self::Email(x) => CSource::WithCustomReadFilter(x.parse(external)?),
		})
	}
}
