/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::{entry::Entry, sink::Stdout};

pub async fn transform(entry: &Entry) {
	Stdout
		.send(entry.msg.clone(), Some("print transform"))
		.await
		.unwrap();
}
