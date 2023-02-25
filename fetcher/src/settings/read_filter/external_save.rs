/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use fetcher_core::read_filter::{external_save::ExternalSave, ReadFilter};

use async_trait::async_trait;
use once_cell::sync::OnceCell;
use std::{io, path::PathBuf};
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
	async fn save(&mut self, read_filter: &dyn ReadFilter) -> io::Result<()> {
		if let Some(rf_conf) =
			fetcher_config::jobs::read_filter::ReadFilter::unparse(read_filter).await
		{
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

			let s = serde_json::to_string(&rf_conf).unwrap();

			file.set_len(0).await?;
			file.rewind().await?;
			file.write_all(s.as_bytes()).await?;
			file.flush().await?;
		}

		Ok(())
	}
}
