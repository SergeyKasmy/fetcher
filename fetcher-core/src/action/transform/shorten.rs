/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Transform;
use crate::action::transform::result::{
	TransformResult as TrRes, TransformedEntry, TransformedMessage,
};
use crate::entry::Entry;

use std::convert::Infallible;
use std::iter::repeat;

#[derive(Debug)]
pub struct Shorten {
	pub len: usize,
}

impl Transform for Shorten {
	type Error = Infallible;

	fn transform(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		Ok(vec![self.transform_impl(entry)])
	}
}

impl Shorten {
	pub fn transform_impl(&self, entry: &Entry) -> TransformedEntry {
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
