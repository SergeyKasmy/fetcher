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

#[derive(Debug)]
pub enum Trim {
	Title,
	Body,
	All,
}

impl Transform for Trim {
	type Error = Infallible;

	fn transform(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		Ok(vec![self.transform_impl(entry)])
	}
}

impl Trim {
	pub fn transform_impl(&self, entry: &Entry) -> TransformedEntry {
		TransformedEntry {
			msg: TransformedMessage {
				title: TrRes::New(
					entry
						.msg
						.title
						.as_deref()
						.map(|s| trim(s, self.should_trim_title())),
				),
				body: TrRes::New(
					entry
						.msg
						.body
						.as_deref()
						.map(|s| trim(s, self.should_trim_body())),
				),
				..Default::default()
			},
			..Default::default()
		}
	}

	fn should_trim_title(&self) -> bool {
		matches!(self, Self::Title | Self::All)
	}

	fn should_trim_body(&self) -> bool {
		matches!(self, Self::Body | Self::All)
	}
}

fn trim(s: &str, should_trim: bool) -> String {
	if should_trim { s.trim() } else { s }.to_owned()
}
