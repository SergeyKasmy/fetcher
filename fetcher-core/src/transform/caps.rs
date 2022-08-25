/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::entry::Entry;
use crate::sink::Message;

#[derive(Debug)]
pub struct Caps;

impl Caps {
	#[tracing::instrument(skip_all)]
	pub fn transform(&self, entry: &Entry) -> Vec<Entry> {
		vec![Entry {
			id: None,
			msg: Message {
				title: entry.msg.title.as_ref().map(|s| s.to_uppercase()),
				body: entry.msg.body.to_uppercase(),
				link: None,
				media: None,
			},
		}]
	}
}
