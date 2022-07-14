/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
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
			!self
				.read_list
				.iter()
				.any(|(read_elem_id, _)| read_elem_id == &elem.id)
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
