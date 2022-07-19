/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::entry::Entry;

#[derive(Debug)]
pub struct Newer {
	pub last_read_id: Option<String>,
}

impl Newer {
	pub(crate) fn new() -> Self {
		Self { last_read_id: None }
	}

	pub(crate) fn last_read(&self) -> Option<&str> {
		self.last_read_id.as_deref()
	}

	/// Make sure the list is sorted newest to oldest
	pub(crate) fn remove_read_from(&self, list: &mut Vec<Entry>) {
		if let Some(last_read_id) = &self.last_read_id {
			if let Some(last_read_id_pos) = list.iter().position(|x| {
				let id = match &x.id {
					Some(id) => id,
					None => return false,
				};

				last_read_id == id
			}) {
				list.drain(last_read_id_pos..);
			}
		}
	}

	/// Check if `current_id` is unread
	/// Make sure `id_list` is sorted newest to oldest
	#[allow(dead_code)] // TODO
	fn is_unread(&self, current_id: &str, id_list: &[&str]) -> bool {
		if let Some(last_read_id) = &self.last_read_id {
			if current_id == last_read_id {
				return false;
			}
			// None => Nether current id nor last read id is first
			// Some(true) => current id is is_unread
			// Some(false) => current id is read
			return id_list
				.iter()
				.fold(None, |acc, &x| match acc {
					None => {
						if x == current_id {
							Some(true)
						} else if x == last_read_id {
							Some(false)
						} else {
							None
						}
					}
					some => some,
				})
				.expect("current_id not found in id_list"); // either FIXME: or write a better comment why it's safe or smth
		}

		true
	}

	pub(crate) fn mark_as_read(&mut self, id: &str) {
		self.last_read_id = Some(id.to_owned());
	}
}

#[cfg(test)]
mod tests {
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
	fn remove_read() {
		let mut rf = Newer::new();
		rf.mark_as_read("3");

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
				id: None,
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
			&[None, Some("5"), Some("4"), None, Some("0"), Some("1")]
		);
	}
}
