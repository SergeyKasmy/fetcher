/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::entry::Entry;

#[derive(Debug)]
pub struct Take {
	pub from: TakeFrom,
	pub num: usize,
}

#[derive(Debug)]
pub enum TakeFrom {
	Beginning,
	End,
}

impl Take {
	pub fn filter(&self, entries: &mut Vec<Entry>) {
		match self.from {
			TakeFrom::Beginning => {
				entries.truncate(self.num);
			}
			TakeFrom::End => {
				let first = entries.len() - self.num;
				entries.drain(first..);
			}
		}
	}
}
