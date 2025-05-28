/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! An error that happened while sending to a sink

use crate::error::InvalidUrlError;
pub use crate::exec::ExecError;

use std::{error::Error as StdError, fmt::Debug, num::TryFromIntError};

#[expect(missing_docs, reason = "error message is self-documenting")]
#[derive(thiserror::Error, Debug)]
pub enum SinkError {
	#[error(transparent)]
	InvalidUrl(#[from] InvalidUrlError),

	#[error("Invalid message ID type. It has probably been copied from an incompatible sink type")]
	InvalidMessageIdType(#[from] TryFromIntError),

	#[cfg(feature = "sink-telegram")]
	#[error("Can't send via Telegram. Message contents: {msg:?}")]
	Telegram {
		source: teloxide::RequestError,
		msg: Box<dyn Debug + Send + Sync>,
	},

	#[cfg(feature = "sink-discord")]
	#[error("Can't send via Discord. Message contents: {msg:?}")]
	Discord {
		source: serenity::Error,
		msg: Box<dyn Debug + Send + Sync>,
	},

	#[error("Can't pass message to a process")]
	Exec(#[from] ExecError),

	#[error("Error writing to stdout")]
	Stdout(#[source] std::io::Error),
}

impl SinkError {
	// TODO: rename to is_network_related, make it a trait and add it to all Error*::Other variants
	// TODO: also, make an Other variant for SinkError
	pub(crate) fn is_connection_err(&self) -> Option<&(dyn StdError + Send + Sync)> {
		match self {
			#[cfg(feature = "sink-telegram")]
			SinkError::Telegram {
				source: teloxide::RequestError::Network(_),
				..
			} => Some(self),
			_ => None,
		}
	}
}
