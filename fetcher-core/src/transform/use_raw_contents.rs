/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{entry::Entry, sink::Message};

pub fn transform(entry: &Entry) -> Entry {
	Entry {
		msg: Message {
			body: entry.raw_contents.clone(),
			..Default::default()
		},
		..Default::default()
	}
}
