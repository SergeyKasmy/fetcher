/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use fetcher_core::source;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Filters {
	sender: Option<String>,
	subjects: Option<Vec<String>>,
	exclude_subjects: Option<Vec<String>>,
}

impl Filters {
	pub(crate) fn parse(self) -> source::email::filters::Filters {
		source::email::filters::Filters {
			sender: self.sender,
			subjects: self.subjects,
			exclude_subjects: self.exclude_subjects,
		}
	}
}
