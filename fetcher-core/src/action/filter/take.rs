/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Take`] filter and the [`TakeFrom`] enum that specifies where the [`Take`] filter should take the entries from

use async_trait::async_trait;

use super::Filter;
use crate::entry::Entry;

/// Take only a set number of entries and discard all others
#[derive(Clone, Debug)]
pub struct Take {
	/// Take from the Beginning or the end of the list?
	pub from: TakeFrom,
	/// Take this number of entries
	pub num: usize,
}

#[expect(missing_docs, reason = "names are self-documenting")]
#[derive(Clone, Debug)]
pub enum TakeFrom {
	Beginning,
	End,
}

#[async_trait]
impl Filter for Take {
	async fn filter(&self, entries: &mut Vec<Entry>) {
		match self.from {
			TakeFrom::Beginning => {
				entries.truncate(self.num);
			}
			TakeFrom::End => {
				let first = entries.len() - self.num;
				entries.drain(..first);
			}
		}
	}
}
