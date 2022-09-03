/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::entry::Entry;
use crate::transform::result::{TransformResult as TrRes, TransformedEntry, TransformedMessage};

use std::iter::repeat;

#[derive(Debug)]
pub struct Shorten {
	pub len: usize,
}

impl Shorten {
	pub fn transform(&self, entry: &Entry) -> TransformedEntry {
		let body = if self.len == 0 {
			TrRes::Empty
		} else {
			TrRes::New(entry.msg.body.clone().map(|s| {
				s.chars()
					.take(self.len)
					.chain(repeat('.').take(3))
					.collect()
			}))
		};

		TransformedEntry {
			msg: TransformedMessage {
				body,
				..Default::default()
			},
			..Default::default()
		}
	}
}
