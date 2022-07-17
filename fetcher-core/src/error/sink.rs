/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::Debug;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Error writing to stdout")]
	StdoutWrite(#[source] std::io::Error),

	#[error("Can't send via Telegram. Message contents: {msg:?}")]
	Telegram {
		source: teloxide::RequestError,
		msg: Box<dyn Debug + Send + Sync>,
	},
}
