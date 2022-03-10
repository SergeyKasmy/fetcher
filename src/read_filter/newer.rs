/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use super::Id;

#[derive(Default, Debug)]
pub struct Newer {
	pub(crate) last_read_id: Option<String>,
}

impl Newer {
	// pub fn new(last_read_id: Option<String>) -> Self {
	// 	Self { last_read_id }
	// }

	pub fn last_read(&self) -> Option<&str> {
		self.last_read_id.as_deref()
	}

	/// Make sure list is sorted newest to oldest
	pub fn remove_read_from<T: Id>(&self, list: &mut Vec<T>) {
		if let Some(last_read_id) = &self.last_read_id {
			if let Some(last_read_id_pos) =
				list.iter().position(|x| x.id() == last_read_id.as_str())
			{
				list.drain(last_read_id_pos..);
			}
		}
	}

	/// Check if `current_id` is unread
	/// Make sure `id_list` is sorted newest to oldest
	pub fn is_unread(&self, current_id: &str, id_list: &[&str]) -> bool {
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
				.expect("current_id not found in id_list");
		}

		true
	}

	pub fn mark_as_read(&mut self, id: &str) {
		self.last_read_id = Some(id.to_owned());
	}
}
