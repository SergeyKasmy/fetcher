/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::entry::Entry;
use crate::transform::result::TransformResult;
use crate::transform::result::TransformedEntry;
use crate::transform::result::TransformedMessage;

#[tracing::instrument(skip_all)]
pub fn transform(entry: &Entry) -> TransformedEntry {
	TransformedEntry {
		msg: TransformedMessage {
			title: TransformResult::New(entry.msg.title.as_ref().map(|s| s.to_uppercase())),
			body: TransformResult::New(entry.msg.body.as_ref().map(|s| s.to_uppercase())),
			..Default::default()
		},
		..Default::default()
	}
}
