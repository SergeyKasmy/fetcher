/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::any::Any;

use async_trait::async_trait;

use crate::{action::filter::Filter, entry::Entry, error::Error, source::MarkAsRead};

use super::ReadFilter;

/// Read Filter that stores the id of the last read entry
#[derive(Clone, Debug)]
pub struct Newer {
	/// the id of the last read entry. None means there haven't been any entries read and thus all entries run through [`filter()`](`Newer::filter()`) will be retained
	pub last_read_id: Option<String>,
}

impl Newer {
	/// Creates a new empty [`Newer`] Read Filter
	#[must_use]
	pub fn new() -> Self {
		Self { last_read_id: None }
	}

	/*
	/// Marks the `id` as already read
	pub fn mark_as_read(&mut self, id: &str) {
		self.last_read_id = Some(id.to_owned());
	}

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
	pub fn remove_read_from(&self, list: &mut Vec<Entry>) {
		if let Some(last_read_id) = &self.last_read_id {
			if let Some(last_read_id_pos) = list.iter().position(|x| {
				let Some(id) = &x.id else { return false };

				last_read_id == id
			}) {
				list.drain(last_read_id_pos..);
			}
		}
	}
	*/

	/// Returns the last read entry id, if any
	#[must_use]
	pub fn last_read(&self) -> Option<&str> {
		self.last_read_id.as_deref()
	}
}

#[async_trait]
impl ReadFilter for Newer {
	/// Doesn't preserve external saving functionality, just copies the data
	async fn as_any(&self) -> Box<dyn Any> {
		Box::new(self.clone())
	}
}

#[async_trait]
impl MarkAsRead for Newer {
	async fn mark_as_read(&mut self, id: &str) -> Result<(), Error> {
		self.last_read_id = Some(id.to_owned());
		let self_clone = self.clone();

		Ok(())
	}

	async fn set_read_only(&mut self) {
		// NOOP
	}
}

#[async_trait]
impl Filter for Newer {
	async fn filter(&self, entries: &mut Vec<Entry>) {
		if let Some(last_read_id) = &self.last_read_id {
			if let Some(last_read_id_pos) = entries.iter().position(|x| {
				let Some(id) = &x.id else { return false };

				last_read_id == id
			}) {
				entries.drain(last_read_id_pos..);
			}
		}
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

	#[test]
	fn mark_as_read() {
		let mut rf = Newer::new();

		rf.mark_as_read("13");
		assert_eq!(rf.last_read_id.as_deref().unwrap(), "13");

		rf.mark_as_read("1002");
		assert_eq!(rf.last_read_id.as_deref().unwrap(), "1002");
	}

	#[test]
	fn last_read() {
		let mut rf = Newer::new();
		assert_eq!(None, rf.last_read());

		rf.mark_as_read("0");
		rf.mark_as_read("1");
		rf.mark_as_read("2");
		assert_eq!(Some("2"), rf.last_read());

		rf.mark_as_read("4");
		assert_eq!(Some("4"), rf.last_read());

		rf.mark_as_read("100");
		rf.mark_as_read("101");
		rf.mark_as_read("200");
		assert_eq!(Some("200"), rf.last_read());
	}

	#[test]
	fn remove_read_long_list() {
		let mut rf = Newer::new();
		rf.mark_as_read("3");

		let mut entries = vec![
			Entry {
				id: None,
				..Default::default()
			},
			Entry {
				id: Some("5".to_owned()),
				..Default::default()
			},
			Entry {
				id: Some("4".to_owned()),
				..Default::default()
			},
			Entry {
				id: None,
				..Default::default()
			},
			Entry {
				id: Some("0".to_owned()),
				..Default::default()
			},
			Entry {
				id: Some("1".to_owned()),
				..Default::default()
			},
			Entry {
				id: Some("3".to_owned()),
				..Default::default()
			},
			Entry {
				id: None,
				..Default::default()
			},
			Entry {
				id: Some("6".to_owned()),
				..Default::default()
			},
			Entry {
				id: Some("8".to_owned()),
				..Default::default()
			},
		];

		rf.filter(&mut entries);

		// remove msgs
		let entries = entries.iter().map(|e| e.id.as_deref()).collect::<Vec<_>>();
		assert_eq!(
			&entries,
			&[None, Some("5"), Some("4"), None, Some("0"), Some("1")]
		);
	}

	#[test]
	fn remove_read_single_different() {
		let mut rf = Newer::new();
		rf.mark_as_read("3");

		let mut entries = vec![Entry {
			id: Some("1".to_owned()),
			..Default::default()
		}];

		rf.filter(&mut entries);

		// remove msgs
		let entries_ids = entries.iter().map(|e| e.id.as_deref()).collect::<Vec<_>>();
		assert_eq!(&entries_ids, &[Some("1")]);
	}

	#[test]
	fn remove_read_single_same() {
		let mut rf = Newer::new();
		rf.mark_as_read("1");

		let mut entries = vec![Entry {
			id: Some("1".to_owned()),
			..Default::default()
		}];
		rf.filter(&mut entries);

		let entries_ids = entries.iter().map(|e| e.id.as_deref()).collect::<Vec<_>>();
		assert_eq!(&entries_ids, &[]);
	}
}
