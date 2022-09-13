/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{entry::Entry, sink::Stdout};

use std::fmt::Write as _;

pub async fn print(entry: &Entry) {
	let mut msg = entry.msg.clone();

	// append raw_contents to help in debugging
	msg.body = {
		let mut body = msg.body.unwrap_or_default();
		let _ = write!(
			body,
			"\n\nid: {:?}\n\nraw_contents: {:?}",
			entry.id, entry.raw_contents
		);
		Some(body)
	};

	Stdout.send(msg, Some("print transform")).await.unwrap();
}
