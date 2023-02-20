/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source::{WithSharedRF as CWithSharedRF, WithSharedRFKind as CWithSharedRFKind};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct StringSource(#[serde_as(deserialize_as = "OneOrMany<_>")] pub Vec<String>);

impl StringSource {
	pub fn parse(self) -> CWithSharedRF {
		let string_sources = self.0.into_iter().map(CWithSharedRFKind::String).collect();

		CWithSharedRF::new(string_sources)
			.expect("should always be the same since we are deserializing only String here")
	}
}