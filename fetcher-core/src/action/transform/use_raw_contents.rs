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
pub struct UseRawContents;

impl Transform for UseRawContents {
	type Error = Infallible;

	fn transform(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		Ok(vec![transform_impl(entry)])
	}
}

pub fn transform_impl(entry: &Entry) -> TransformedEntry {
	TransformedEntry {
		msg: TransformedMessage {
			body: TrRes::New(entry.raw_contents.clone()),
			..Default::default()
		},
		..Default::default()
	}
}
