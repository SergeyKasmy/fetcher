/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! This module contains the [`Stdout`] sink

use crate::sinks::{Message, Sink, error::SinkError};

use tokio::io::{self, AsyncWriteExt};

use super::MessageId;

/// Print message to stdout. Mostly used for debugging
#[derive(Debug)]
pub struct Stdout;

impl Sink for Stdout {
	/// Prints a message with an optional tag to stdout
	///
	/// # Errors
	/// if there was an error writing to stdout
	async fn send(
		&self,
		msg: &Message,
		_reply_to: Option<&MessageId>,
		tag: Option<&str>,
	) -> Result<Option<MessageId>, SinkError> {
		io::stdout().write_all(format!(
			"------------------------------\nMessage:\nTitle: {title}\n\nBody:\n{body}\n\nLink: {link}\n\nMedia: {media:?}\n\nTag: {tag:?}\n------------------------------\n",
			title = msg.title.as_deref().unwrap_or("None"),
			body = msg.body.as_deref().unwrap_or("None"),
			link = msg.link.as_ref().map(|url| url.as_str().to_owned()).as_deref().unwrap_or("None"),
			media = msg.media,
			tag = tag.unwrap_or("None")
		).as_bytes()).await.map_err(SinkError::Stdout)?;

		Ok(None)
	}
}
