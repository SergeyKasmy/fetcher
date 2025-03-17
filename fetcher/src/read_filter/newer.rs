/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use async_trait::async_trait;

use super::{MarkAsRead, ReadFilter};
use crate::{
	action::filters::Filter,
	entry::{Entry, EntryId},
	error::FetcherError,
};

/// Read Filter that stores the id of the last read entry
#[derive(Clone, Debug)]
pub struct Newer {
	/// the id of the last read entry. None means there haven't been any entries read and thus all entries run through [`filter()`](`Newer::filter()`) will be retained
	pub last_read_id: Option<EntryId>,
}

impl Newer {
	/// Creates a new empty [`Newer`] Read Filter
	#[must_use]
	pub const fn new() -> Self {
		Self { last_read_id: None }
	}

	/// Returns the last read entry id, if any
	#[must_use]
	pub const fn last_read(&self) -> Option<&EntryId> {
		self.last_read_id.as_ref()
	}
}

#[async_trait]
impl ReadFilter for Newer {}

#[async_trait]
impl MarkAsRead for Newer {
	async fn mark_as_read(&mut self, id: &EntryId) -> Result<(), FetcherError> {
		self.last_read_id = Some(id.clone());
		Ok(())
	}

	async fn set_read_only(&mut self) {
		// NOOP
	}
}

#[async_trait]
impl Filter for Newer {
	/// Removes all entries that are in the `list` after the last one read, including itself, in order
	/// Note: Make sure the list is sorted newest to oldest
	///
	/// # Example:
	/// Last one marked as read: id 5
	/// Entry list:
	/// * id: 9
	/// * id: 8
	/// * id: 3
	/// * id: 5
	/// * id: 7
	/// * id: 2
	///
	/// Entry list after running through [`Newer`]:
	/// * id 9
	/// * id 8
	/// * id 3
	#[tracing::instrument(level = "debug", name = "filter_read", skip_all)]
	async fn filter(&self, entries: &mut Vec<Entry>) {
		if let Some(last_read_id) = &self.last_read_id {
			if let Some(last_read_id_pos) = entries.iter().position(|x| {
				let Some(id) = &x.id else { return false };

				last_read_id == id
			}) {
				let removed_elems = entries.drain(last_read_id_pos..).count();
				tracing::debug!("Removed {removed_elems} already read entries");
				tracing::trace!("Unread entries remaining: {entries:#?}");
			}
		}
	}

	fn is_readfilter(&self) -> bool {
		true
	}
}

impl Default for Newer {
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
		let mut rf = Newer::new();

		rf.mark_as_read(&"13".into()).await.unwrap();
		assert_eq!(rf.last_read_id.as_deref().unwrap(), "13");

		rf.mark_as_read(&"1002".into()).await.unwrap();
		assert_eq!(rf.last_read_id.as_deref().unwrap(), "1002");
	}

	#[tokio::test]
	async fn last_read() {
		let mut rf = Newer::new();
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
	async fn remove_read_long_list() {
		let mut rf = Newer::new();
		rf.mark_as_read(&"3".into()).await.unwrap();

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
				id: None,
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

		rf.filter(&mut entries).await;

		// remove msgs
		let entries = entries.iter().map(|e| e.id.as_deref()).collect::<Vec<_>>();
		assert_eq!(
			&entries,
			&[None, Some("5"), Some("4"), None, Some("0"), Some("1")]
		);
	}

	#[tokio::test]
	async fn remove_read_single_different() {
		let mut rf = Newer::new();
		rf.mark_as_read(&"3".into()).await.unwrap();

		let mut entries = vec![Entry {
			id: Some("1".into()),
			..Default::default()
		}];

		rf.filter(&mut entries).await;

		// remove msgs
		let entries_ids = entries.iter().map(|e| e.id.as_deref()).collect::<Vec<_>>();
		assert_eq!(&entries_ids, &[Some("1")]);
	}

	#[tokio::test]
	async fn remove_read_single_same() {
		let mut rf = Newer::new();
		rf.mark_as_read(&"1".into()).await.unwrap();

		let mut entries = vec![Entry {
			id: Some("1".into()),
			..Default::default()
		}];
		rf.filter(&mut entries).await;

		let entries_ids = entries.iter().map(|e| e.id.as_deref()).collect::<Vec<_>>();
		assert_eq!(&entries_ids, &[]);
	}
}
