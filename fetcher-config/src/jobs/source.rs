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

use self::{
	email::Email, exec::Exec, file::File, http::Http, reddit::Reddit, string::StringSource,
};
use crate::{FetcherConfigError, jobs::external_data::ProvideExternalData};
use fetcher_core::{
	read_filter::ReadFilter as CReadFilter,
	source::{
		Source as CSource, SourceWithSharedRF as CSourceWithSharedRF,
		always_errors::AlwaysErrors as CAlwaysErrors,
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
	File(File),
	Reddit(Reddit),
	Exec(Exec),

	// with custom read filter
	Email(Email),
	AlwaysErrors,
}

impl Source {
	pub fn decode_from_conf<RF, D>(
		self,
		rf: Option<RF>,
		external: &D,
	) -> Result<Box<dyn CSource>, FetcherConfigError>
	where
		RF: CReadFilter + 'static,
		D: ProvideExternalData + ?Sized,
	{
		// make a dyn CSourceWithSharedRF out of a CFetch and the read filter parameter
		macro_rules! with_read_filter {
			($source:expr) => {
				Box::new(CSourceWithSharedRF {
					source: $source,
					rf,
				})
			};
		}

		Ok(match self {
			// with shared read filter
			Self::String(x) => with_read_filter!(x.decode_from_conf()),
			Self::Http(x) => with_read_filter!(x.decode_from_conf()?),
			Self::File(x) => with_read_filter!(x.decode_from_conf()),
			Self::Reddit(x) => with_read_filter!(x.decode_from_conf()),
			Self::Exec(x) => with_read_filter!(x.decode_from_conf()),

			// with custom read filter
			Self::Email(x) => Box::new(x.decode_from_conf(external)?),
			Self::AlwaysErrors => Box::new(CAlwaysErrors),
		})
	}

	#[must_use]
	pub fn supports_replies(&self) -> bool {
		// Source::Email will support replies in the future
		#[expect(
			clippy::match_single_binding,
			reason = "will be easy to add new \"true\" arms in the future"
		)]
		match self {
			_ => false,
		}
	}
}
