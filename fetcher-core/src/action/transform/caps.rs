/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::Transform;
use crate::action::transform::result::TransformResult;
use crate::action::transform::result::TransformedEntry;
use crate::action::transform::result::TransformedMessage;
use crate::entry::Entry;

use std::convert::Infallible;

#[derive(Debug)]
pub struct Caps;

fn transform_impl(entry: &Entry) -> TransformedEntry {
	TransformedEntry {
		msg: TransformedMessage {
			title: TransformResult::New(entry.msg.title.as_ref().map(|s| s.to_uppercase())),
			body: TransformResult::New(entry.msg.body.as_ref().map(|s| s.to_uppercase())),
			..Default::default()
		},
		..Default::default()
	}
}

impl Transform for Caps {
	type Error = Infallible;

	#[tracing::instrument(skip_all)]
	fn transform(&self, entry: &Entry) -> Result<Vec<TransformedEntry>, Self::Error> {
		Ok(vec![transform_impl(entry)])
	}
}
