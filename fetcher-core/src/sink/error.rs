/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! An error that happened while sending to a sink

pub use crate::exec::ExecError;

use std::fmt::Debug;

#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum SinkError {
	#[error("Can't send via Telegram. Message contents: {msg:?}")]
	Telegram {
		source: teloxide::RequestError,
		msg: Box<dyn Debug + Send + Sync>,
	},

	#[error("Can't pass message to a process")]
	Exec(#[from] ExecError),

	#[error("Error writing to stdout")]
	Stdout(#[source] std::io::Error),
}
