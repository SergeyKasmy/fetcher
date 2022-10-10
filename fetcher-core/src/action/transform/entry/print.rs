/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains a debug print transform-like function [`print()`]

use crate::{entry::Entry, sink::Stdout};

use std::fmt::Write as _;

/// Debug prints current entry contents
pub async fn print(entry: &Entry) {
	let mut msg = entry.msg.clone();

	// append id and raw_contents entry fields to the body to help in debugging
	msg.body = {
		let mut body = msg.body.unwrap_or_else(|| "None".to_owned());
		let _ = write!(
			body,
			"\n\nid: {:?}\n\nraw_contents: {:?}",
			entry.id, entry.raw_contents
		);
		Some(body)
	};

	Stdout
		.send(msg, Some("print transform"))
		.await
		.expect("stdout is unavailable");
}
