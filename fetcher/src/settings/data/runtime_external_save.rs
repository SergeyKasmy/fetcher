/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod entry_to_msg_map;
pub mod read_filter;

use fetcher_core::{
	entry::EntryId, external_save::ExternalSave, read_filter::ReadFilter, sink::message::MessageId,
};

use async_trait::async_trait;
use once_cell::sync::OnceCell;
use std::{collections::HashMap, io, path::PathBuf};
use tokio::{
	fs,
	io::{AsyncSeekExt, AsyncWriteExt},
};

#[derive(Debug)]
pub struct TruncatingFileWriter {
	path: PathBuf,
	file: OnceCell<fs::File>,
}

impl TruncatingFileWriter {
	#[must_use]
	pub fn new(path: PathBuf) -> Self {
		Self {
			path,
			file: OnceCell::new(),
		}
	}
}

#[async_trait]
impl ExternalSave for TruncatingFileWriter {
	async fn save_read_filter(&mut self, read_filter: &dyn ReadFilter) -> io::Result<()> {
		if let Some(rf_conf) =
			fetcher_config::jobs::read_filter::ReadFilter::unparse(read_filter).await
		{
			let s = serde_json::to_string(&rf_conf).unwrap();

			self.write(s.as_bytes()).await?;
		}

		Ok(())
	}

	async fn save_entry_to_msg_map(&mut self, map: &HashMap<EntryId, MessageId>) -> io::Result<()> {
		let map_conf =
			fetcher_config::jobs::task::entry_to_msg_map::EntryToMsgMap::unparse(map.clone());
		let s = serde_json::to_string(&map_conf).unwrap();

		self.write(s.as_bytes()).await
	}
}

// FIXME: ExternalSave(Os { code: 17, kind: AlreadyExists, message: "File exists" }) when running in mark-old-as-read mode. I'm pretty sure it happens here...
impl TruncatingFileWriter {
	async fn write(&mut self, data: &[u8]) -> io::Result<()> {
		// create file just before writing
		if self.file.get().is_none() {
			if let Some(parent) = self.path.parent() {
				fs::create_dir_all(parent).await?;
			}

			let file = fs::OpenOptions::new()
				.create(true)
				.write(true)
				.open(&self.path)
				.await?;

			self.file
				.set(file)
				.expect("file should be none because .get().is_none() is true");
		}

		let file = self
			.file
			.get_mut()
			.expect("file should exist since it should've been set right up above");

		file.set_len(0).await?;
		file.rewind().await?;
		file.write_all(data).await?;
		file.flush().await?;

		Ok(())
	}
}
