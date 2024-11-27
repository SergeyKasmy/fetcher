/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod entry_to_msg_map;
pub mod read_filter;

use fetcher_core::{
	entry::EntryId,
	external_save::{ExternalSave, ExternalSaveError},
	read_filter::ReadFilter,
	sink::message::MessageId,
};

use async_trait::async_trait;
use core::fmt;
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

#[derive(Debug)]
pub struct DisplayPath(pub PathBuf);

impl TruncatingFileWriter {
	#[must_use]
	pub const fn new(path: PathBuf) -> Self {
		Self {
			path,
			file: OnceCell::new(),
		}
	}
}

#[async_trait]
impl ExternalSave for TruncatingFileWriter {
	async fn save_read_filter(
		&mut self,
		read_filter: &dyn ReadFilter,
	) -> Result<(), ExternalSaveError> {
		if let Some(rf_conf) =
			fetcher_config::jobs::read_filter::ReadFilter::encode_into_conf(read_filter).await
		{
			let s = serde_json::to_string(&rf_conf)
				.expect("A ReadFilter should always be serializable");

			self.write(s.as_bytes())
				.await
				.map_err(|source| ExternalSaveError {
					source,
					path: Some(Box::new(DisplayPath(self.path.clone()))),
				})?;
		}

		Ok(())
	}

	async fn save_entry_to_msg_map(
		&mut self,
		map: &HashMap<EntryId, MessageId>,
	) -> Result<(), ExternalSaveError> {
		let map_conf =
			fetcher_config::jobs::task::entry_to_msg_map::EntryToMsgMap::encode_into_conf(
				map.clone(),
			);
		let s = serde_json::to_string(&map_conf)
			.expect("An EntryToMsgMap should always be serializable");

		self.write(s.as_bytes())
			.await
			.map_err(|source| ExternalSaveError {
				source,
				path: Some(Box::new(DisplayPath(self.path.clone()))),
			})
	}
}

impl TruncatingFileWriter {
	async fn write(&mut self, data: &[u8]) -> io::Result<()> {
		// create file just before writing
		if self.file.get().is_none() {
			if let Some(parent) = self.path.parent() {
				fs::create_dir_all(parent).await?;
			}

			let file = fs::OpenOptions::new()
				.create(true)
				.truncate(true)
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

impl fmt::Display for DisplayPath {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0.display())
	}
}
