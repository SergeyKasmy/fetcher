/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::source::{
	Exec as CExec, WithSharedRF as CWithSharedRF, WithSharedRFKind as CWithSharedRFKind,
};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, OneOrMany};

#[serde_as]
#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct Exec {
	#[serde_as(deserialize_as = "OneOrMany<_>")]
	pub cmd: Vec<String>,
}

impl Exec {
	pub fn parse(self) -> CWithSharedRF {
		let exec_sources = self
			.cmd
			.into_iter()
			.map(|cmd| CWithSharedRFKind::Exec(CExec { cmd }))
			.collect();

		CWithSharedRF::new(exec_sources)
			.expect("should always be the same since we are deserializing only Exec here")
	}
}
