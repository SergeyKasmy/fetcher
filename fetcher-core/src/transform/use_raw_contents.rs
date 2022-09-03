/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::entry::Entry;
use crate::transform::result::{TransformResult as TrRes, TransformedEntry, TransformedMessage};

pub fn transform(entry: &Entry) -> TransformedEntry {
	TransformedEntry {
		msg: TransformedMessage {
			body: TrRes::New(entry.raw_contents.clone()),
			..Default::default()
		},
		..Default::default()
	}
}
