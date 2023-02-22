/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod email;
pub mod exec;
pub mod file;
pub mod http;
pub mod reddit;
pub mod string;
pub mod twitter;

use self::{
	email::Email, exec::Exec, file::File, http::Http, reddit::Reddit, string::StringSource,
	twitter::Twitter,
};
use crate::{jobs::external_data::ProvideExternalData, Error};
use fetcher_core::{
	read_filter::ReadFilter as CReadFilter,
	source::{Source as CSource, SourceWithSharedRF as CSourceWithSharedRF},
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
	// with shared read filter
	String(StringSource),
	Http(Http),
	Twitter(Twitter),
	File(File),
	Reddit(Reddit),
	Exec(Exec),

	// with custom read filter
	Email(Email),
}

impl Source {
	pub fn parse(
		self,
		rf: Option<Arc<RwLock<dyn CReadFilter>>>,
		external: &dyn ProvideExternalData,
	) -> Result<Box<dyn CSource>, Error> {
		// make a dyn CSourceWithSharedRF out of a dyn CFetch and the read filter parameter
		macro_rules! WithSharedRF {
			($source:expr) => {
				Box::new(CSourceWithSharedRF {
					source: $source,
					rf: rf.map(|x| Box::new(x) as Box<_>),
				})
			};
		}

		Ok(match self {
			Self::String(x) => WithSharedRF!(x.parse()),
			Self::Http(x) => WithSharedRF!(x.parse()?),
			Self::Twitter(x) => WithSharedRF!(x.parse(external)?),
			Self::File(x) => WithSharedRF!(x.parse()),
			Self::Reddit(x) => WithSharedRF!(x.parse()),
			Self::Exec(x) => WithSharedRF!(x.parse()),
			Self::Email(x) => x.parse(external)?,
		})
	}
}
