/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Take`] filter and the [`TakeFrom`] enum that specifies where the [`Take`] filter should take the entries from

use std::convert::Infallible;

use super::{Filter, FilterableEntries};

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

impl Filter for Take {
	type Err = Infallible;

	async fn filter(&mut self, mut entries: FilterableEntries<'_>) -> Result<(), Self::Err> {
		match self.from {
			TakeFrom::Beginning => {
				entries.truncate(self.num);
			}
			TakeFrom::End => {
				let first = entries.len() - self.num;
				entries.drain(..first);
			}
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::{Take, TakeFrom};
	use crate::{
		actions::filters::{Filter, FilterableEntries},
		entry::Entry,
		sinks::message::Message,
	};

	#[tokio::test]
	async fn beginning() {
		let mut entries = (0..5)
			.map(|idx| {
				Entry::builder()
					.msg(Message::builder().body(idx.to_string()))
					.build()
			})
			.collect::<Vec<_>>();

		let mut take = Take {
			from: TakeFrom::Beginning,
			num: 2,
		};

		take.filter(FilterableEntries::new(&mut entries))
			.await
			.unwrap();

		assert_eq!(
			entries
				.iter()
				.map(|ent| ent.msg.body.as_ref().unwrap().as_str())
				.collect::<Vec<_>>(),
			["0", "1"]
		);
	}

	#[tokio::test]
	async fn end() {
		let mut entries = (0..5)
			.map(|idx| {
				Entry::builder()
					.msg(Message::builder().body(idx.to_string()))
					.build()
			})
			.collect::<Vec<_>>();

		let mut take = Take {
			from: TakeFrom::End,
			num: 2,
		};

		take.filter(FilterableEntries::new(&mut entries))
			.await
			.unwrap();

		assert_eq!(
			entries
				.iter()
				.map(|ent| ent.msg.body.as_ref().unwrap().as_str())
				.collect::<Vec<_>>(),
			["3", "4"]
		);
	}
}
