/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::iter::repeat;

use crate::{entry::Entry, sink::Message};

#[derive(Debug)]
pub struct Shorten {
	pub len: usize,
}

impl Shorten {
	pub fn transform(&self, entry: &Entry) -> Entry {
		let body = if self.len == 0 {
			Some(String::new())
		} else {
			entry.msg.body.clone().map(|s| {
				s.chars()
					.take(self.len)
					.chain(repeat('.').take(3))
					.collect()
			})
		};

		Entry {
			msg: Message {
				body,
				..Default::default()
			},
			..Default::default()
		}
	}
}
