/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::{MarkAsRead, ReadFilter};
use crate::{
	action::filters::Filter,
	entry::{Entry, EntryId},
	error::FetcherError,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, convert::Infallible};

const MAX_LIST_LEN: usize = 500;

/// Read Filter that stores a list of all entries read
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotPresent {
	read_list: VecDeque<(EntryId, DateTime<Utc>)>,
}

impl NotPresent {
	/// Creates a new empty [`NotPresent`] Read Filter
	#[must_use]
	pub fn new() -> Self {
		Self {
			read_list: VecDeque::default(),
		}
	}

	/// Returns the id of the last read entry, if any
	#[must_use]
	pub fn last_read(&self) -> Option<&EntryId> {
		self.read_list.back().map(|(s, _)| s)
	}

	/// Checks if the `id` is unread
	#[must_use]
	pub fn is_unread(&self, id: &EntryId) -> bool {
		!self.read_list.iter().any(|(ent_id, _)| ent_id == id)
	}

	/// Provides a read only view into the inner collection
	pub fn iter(&self) -> impl Iterator<Item = &(EntryId, DateTime<Utc>)> {
		self.read_list.iter()
	}

	/// Checks if there wasn't any entry marked as read yet
	#[must_use]
	pub fn is_empty(&self) -> bool {
		self.read_list.is_empty()
	}
}

impl ReadFilter for NotPresent {}

impl MarkAsRead for NotPresent {
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError> {
		self.read_list.push_back((id.clone(), chrono::Utc::now()));

		while self.read_list.len() > MAX_LIST_LEN {
			self.read_list.pop_front();
		}

		Ok(())
	}

	async fn set_read_only(&mut self) {
		// NOOP
	}
}

impl Filter for NotPresent {
	type Error = Infallible;

	#[tracing::instrument(level = "debug", name = "filter_read", skip_all)]
	async fn filter(&self, entries: &mut Vec<Entry>) -> Result<(), Self::Error> {
		let old_len = entries.len();
		entries.retain(|elem| {
			// retain elements with no id
			let Some(id) = &elem.id else { return true };

			!self
				.read_list
				.iter()
				.any(|(read_elem_id, _)| read_elem_id == id)
		});

		let removed_elems = old_len - entries.len();
		tracing::debug!("Removed {removed_elems} already read entries");
		tracing::trace!("Unread entries remaining: {entries:#?}");

		Ok(())
	}

	fn is_readfilter(&self) -> bool {
		true
	}
}

impl FromIterator<(EntryId, DateTime<Utc>)> for NotPresent {
	fn from_iter<I: IntoIterator<Item = (EntryId, DateTime<Utc>)>>(iter: I) -> Self {
		Self {
			read_list: iter.into_iter().collect(),
		}
	}
}

impl Default for NotPresent {
	fn default() -> Self {
		Self::new()
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;

	#[tokio::test]
	async fn mark_as_read() {
		let mut rf = NotPresent::new();

		rf.mark_as_read(&"13".into()).await.unwrap();
		assert_eq!(
			&rf.read_list.iter().map(|(s, _date)| s).collect::<Vec<_>>(),
			&[&"13".into()]
		);

		rf.mark_as_read(&"1002".into()).await.unwrap();
		assert_eq!(
			&rf.read_list.iter().map(|(s, _date)| s).collect::<Vec<_>>(),
			&[&"13".into(), &"1002".into()]
		);
	}

	#[tokio::test]
	async fn mark_as_read_full_queue() {
		let mut rf = NotPresent::new();
		let mut v = Vec::with_capacity(MAX_LIST_LEN);

		for i in 0..600 {
			let id = EntryId(i.to_string());
			rf.mark_as_read(&id).await.unwrap();
			v.push(id);
		}

		// keep only the last MAX_LIST_LEN elements
		let trimmed_v = v[v.len() - MAX_LIST_LEN..].iter().collect::<Vec<_>>();

		let rf_list = rf.read_list.iter().map(|(s, _date)| s).collect::<Vec<_>>();

		assert_eq!(trimmed_v, rf_list);
	}

	#[tokio::test]
	async fn last_read() {
		let mut rf = NotPresent::new();
		assert_eq!(None, rf.last_read());

		rf.mark_as_read(&"0".into()).await.unwrap();
		rf.mark_as_read(&"1".into()).await.unwrap();
		rf.mark_as_read(&"2".into()).await.unwrap();
		assert_eq!(Some(&"2".into()), rf.last_read());

		rf.mark_as_read(&"4".into()).await.unwrap();
		assert_eq!(Some(&"4".into()), rf.last_read());

		rf.mark_as_read(&"100".into()).await.unwrap();
		rf.mark_as_read(&"101".into()).await.unwrap();
		rf.mark_as_read(&"200".into()).await.unwrap();
		assert_eq!(Some(&"200".into()), rf.last_read());
	}

	#[tokio::test]
	async fn remove_read() {
		let mut rf = NotPresent::new();
		rf.mark_as_read(&"0".into()).await.unwrap();
		rf.mark_as_read(&"1".into()).await.unwrap();
		rf.mark_as_read(&"2".into()).await.unwrap();
		rf.mark_as_read(&"5".into()).await.unwrap();
		rf.mark_as_read(&"7".into()).await.unwrap();

		let mut entries = vec![
			Entry {
				id: None,
				..Default::default()
			},
			Entry {
				id: Some("5".into()),
				..Default::default()
			},
			Entry {
				id: Some("4".into()),
				..Default::default()
			},
			Entry {
				id: Some("0".into()),
				..Default::default()
			},
			Entry {
				id: Some("1".into()),
				..Default::default()
			},
			Entry {
				id: Some("3".into()),
				..Default::default()
			},
			Entry {
				id: None,
				..Default::default()
			},
			Entry {
				id: Some("6".into()),
				..Default::default()
			},
			Entry {
				id: Some("8".into()),
				..Default::default()
			},
		];

		rf.filter(&mut entries).await.unwrap();

		// remove msgs
		let entries = entries.iter().map(|e| e.id.as_deref()).collect::<Vec<_>>();
		assert_eq!(
			&entries,
			&[None, Some("4"), Some("3"), None, Some("6"), Some("8")]
		);
	}
}
