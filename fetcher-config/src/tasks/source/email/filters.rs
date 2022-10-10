/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};

use fetcher_core::source::email::Filters as CFilters;

#[derive(Deserialize, Serialize, Debug)]
pub struct Filters {
	sender: Option<String>,
	subjects: Option<Vec<String>>,
	exclude_subjects: Option<Vec<String>>,
}

impl Filters {
	pub fn parse(self) -> CFilters {
		CFilters {
			sender: self.sender,
			subjects: self.subjects,
			exclude_subjects: self.exclude_subjects,
		}
	}
}
