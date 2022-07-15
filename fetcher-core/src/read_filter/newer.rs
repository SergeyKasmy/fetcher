/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
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
	pub(crate) fn is_unread(&self, current_id: &str, id_list: &[&str]) -> bool {
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
