/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(missing_docs)]

use std::{fmt::Debug, io};

/// An error that happened while sending to a sink
#[allow(missing_docs)] // error message is self-documenting
#[derive(thiserror::Error, Debug)]
pub enum Error {
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

#[derive(thiserror::Error, Debug)]
pub enum ExecError {
	#[error("Can't start the process")]
	CantStart(#[source] io::Error),

	#[error("Can't pass data to the stdin of the process")]
	CantWriteStdin(#[source] io::Error),
}
