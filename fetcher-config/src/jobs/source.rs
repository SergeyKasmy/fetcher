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
	source::{
		always_errors::AlwaysErrors as CAlwaysErrors, Source as CSource,
		SourceWithSharedRF as CSourceWithSharedRF,
	},
};

use serde::{Deserialize, Serialize};

#[allow(clippy::large_enum_variant)]
#[derive(Deserialize, Serialize, Clone, Debug)]
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
	AlwaysErrors,
}

impl Source {
	pub fn parse<RF, D>(self, rf: Option<RF>, external: &D) -> Result<Box<dyn CSource>, Error>
	where
		RF: CReadFilter + 'static,
		D: ProvideExternalData + ?Sized,
	{
		// make a dyn CSourceWithSharedRF out of a CFetch and the read filter parameter
		macro_rules! WithSharedRF {
			($source:expr) => {
				Box::new(CSourceWithSharedRF {
					source: $source,
					rf,
				})
			};
		}

		Ok(match self {
			// with shared read filter
			Self::String(x) => WithSharedRF!(x.parse()),
			Self::Http(x) => WithSharedRF!(x.parse()?),
			Self::Twitter(x) => WithSharedRF!(x.parse(external)?),
			Self::File(x) => WithSharedRF!(x.parse()),
			Self::Reddit(x) => WithSharedRF!(x.parse()),
			Self::Exec(x) => WithSharedRF!(x.parse()),

			// with custom read filter
			Self::Email(x) => Box::new(x.parse(external)?),
			Self::AlwaysErrors => Box::new(CAlwaysErrors),
		})
	}
}
