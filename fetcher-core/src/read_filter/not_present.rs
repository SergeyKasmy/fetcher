/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use chrono::{DateTime, Utc};
use std::collections::VecDeque;

use crate::entry::Entry;

const MAX_LIST_LEN: usize = 500;

#[derive(Debug)]
pub struct NotPresent {
	pub read_list: VecDeque<(String, DateTime<Utc>)>,
}

impl NotPresent {
	pub(crate) fn new() -> Self {
		Self {
			read_list: VecDeque::default(),
		}
	}

	pub(crate) fn last_read(&self) -> Option<&str> {
		self.read_list.back().map(|(s, _)| s.as_str())
	}

	pub(crate) fn remove_read_from(&self, list: &mut Vec<Entry>) {
		list.retain(|elem| {
			// retain elements with no id
			let id = match &elem.id {
				Some(id) => id,
				None => return true,
			};

			!self
				.read_list
				.iter()
				.any(|(read_elem_id, _)| read_elem_id == id)
		});
	}

	pub(crate) fn mark_as_read(&mut self, id: &str) {
		self.read_list
			.push_back((id.to_owned(), chrono::Utc::now()));

		while self.read_list.len() > MAX_LIST_LEN {
			self.read_list.pop_front();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn mark_as_read() {
		let mut rf = NotPresent::new();

		rf.mark_as_read("13");
		assert_eq!(
			&rf.read_list
				.iter()
				.map(|(s, _date)| s.as_str())
				.collect::<Vec<_>>(),
			&["13"]
		);

		rf.mark_as_read("1002");
		assert_eq!(
			&rf.read_list
				.iter()
				.map(|(s, _date)| s.as_str())
				.collect::<Vec<_>>(),
			&["13", "1002"]
		);
	}

	#[test]
	fn mark_as_read_full_queue() {
		let mut rf = NotPresent::new();
		let mut v = Vec::with_capacity(MAX_LIST_LEN);

		for i in 0..600 {
			rf.mark_as_read(&i.to_string());
			v.push(i.to_string());
		}

		// keep only the last MAX_LIST_LEN elements
		let trimmed_v = v[v.len() - MAX_LIST_LEN..]
			.iter()
			.map(String::as_str)
			.collect::<Vec<_>>();

		let rf_list = rf
			.read_list
			.iter()
			.map(|(s, _date)| s.as_str())
			.collect::<Vec<_>>();

		assert_eq!(trimmed_v, rf_list);
	}

	#[test]
	fn last_read() {
		let mut rf = NotPresent::new();
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
	fn remove_read() {
		let mut rf = NotPresent::new();
		rf.mark_as_read("0");
		rf.mark_as_read("1");
		rf.mark_as_read("2");
		rf.mark_as_read("5");
		rf.mark_as_read("7");

		let mut entries = vec![
			Entry {
				id: None,
				msg: crate::sink::Message::default(),
			},
			Entry {
				id: Some("5".to_owned()),
				msg: crate::sink::Message::default(),
			},
			Entry {
				id: Some("4".to_owned()),
				msg: crate::sink::Message::default(),
			},
			Entry {
				id: Some("0".to_owned()),
				msg: crate::sink::Message::default(),
			},
			Entry {
				id: Some("1".to_owned()),
				msg: crate::sink::Message::default(),
			},
			Entry {
				id: Some("3".to_owned()),
				msg: crate::sink::Message::default(),
			},
			Entry {
				id: None,
				msg: crate::sink::Message::default(),
			},
			Entry {
				id: Some("6".to_owned()),
				msg: crate::sink::Message::default(),
			},
			Entry {
				id: Some("8".to_owned()),
				msg: crate::sink::Message::default(),
			},
		];

		rf.remove_read_from(&mut entries);

		// remove msgs
		let entries = entries.iter().map(|e| e.id.as_deref()).collect::<Vec<_>>();
		assert_eq!(
			&entries,
			&[None, Some("4"), Some("3"), None, Some("6"), Some("8")]
		);
	}
}
