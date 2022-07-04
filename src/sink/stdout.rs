/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use tokio::io::AsyncWriteExt;

use crate::error::Error;
use crate::error::Result;
use crate::sink::Message;

#[derive(Debug)]
pub struct Stdout;

impl Stdout {
	pub async fn send(&self, msg: Message, tag: Option<&str>) -> Result<()> {
		tokio::io::stdout().write_all(format!(
			"------------------------------\nMessage:\nTitle: {title}\n\nBody:\n{body}\n\nLink: {link:?}\nMedia: {media}\nTag: {tag}\n------------------------------",
			title = msg.title.as_deref().unwrap_or("None"),
			body = msg.body,
			link = msg.link,
			media = msg.media.is_some(),
			tag = tag.unwrap_or("None")
		).as_bytes()).await.map_err(Error::Stdout)
	}
}