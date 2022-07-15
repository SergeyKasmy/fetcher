/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::{io::Write, path::PathBuf};
use tokio::fs;

use super::PREFIX;
use crate::config;
use crate::error::ConfigError;
use fetcher_core::read_filter::{ExternalSave, ReadFilter};

const READ_DATA_DIR: &str = "read";

fn read_filter_path(name: &str) -> Result<PathBuf, ConfigError> {
	debug_assert!(!name.is_empty());
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/read/{name}"))
	} else {
		xdg::BaseDirectories::with_profile(PREFIX, READ_DATA_DIR)?
			.place_data_file(name)
			.map_err(|e| ConfigError::Read(e, format!("READ_DATA_DIR/{name}").into()))?
	})
}

/// Returns a read filter for the task name from the filesystem.
///
/// # Errors
/// * if the file is inaccessible
/// * if the file is corrupted
#[tracing::instrument(skip(default))]
pub(crate) async fn get(
	name: &str,
	// TODO: remove option
	default: Option<fetcher_core::read_filter::Kind>,
) -> Result<Option<ReadFilter>, ConfigError> {
	struct TruncatingFileWriter {
		file: std::fs::File,
	}

	impl std::io::Write for TruncatingFileWriter {
		fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
			use std::io::Seek;

			self.file.set_len(0)?;
			self.file.rewind()?;
			self.file.write(buf)
		}

		fn flush(&mut self) -> std::io::Result<()> {
			self.file.flush()
		}
	}

	impl ExternalSave for TruncatingFileWriter {
		fn save(
			&mut self,
			read_filter: &fetcher_core::read_filter::ReadFilterInner,
		) -> std::io::Result<()> {
			if let Some(filter_conf) = crate::config::read_filter::ReadFilter::unparse(read_filter)
			{
				let s = serde_json::to_string(&filter_conf).unwrap();
				return self.write_all(s.as_bytes());
			}

			Ok(())
		}
	}

	let writer = || -> Result<TruncatingFileWriter, ConfigError> {
		let path = read_filter_path(name)?;
		if let Some(parent) = path.parent() {
			std::fs::create_dir_all(parent)
				.map_err(|e| ConfigError::Write(e, parent.to_owned()))?;
		}

		let file = std::fs::OpenOptions::new()
			.create(true)
			.write(true)
			.open(&path)
			.map_err(|e| ConfigError::Write(e, path.clone()))?;

		Ok(TruncatingFileWriter { file })
	};

	let path = read_filter_path(name)?;

	let filter = match fs::read_to_string(&path).await.ok() {
		None => {
			tracing::trace!("Read filter save file doesn't exist");
			None
		}
		Some(filter_str) => match filter_str.len() {
			0 => {
				tracing::trace!("Read filter save file exists but is empty");
				None
			}
			l => {
				tracing::trace!("Read filter save file exists and is {} bytes long", l);

				let read_filter_conf: config::read_filter::ReadFilter =
					serde_json::from_str(&filter_str)
						.map_err(|e| ConfigError::CorruptedConfig(Box::new(e), path))?;
				Some(read_filter_conf.parse(Box::new(writer()?)))
			}
		},
	};

	match filter {
		f @ Some(_) => Ok(f),
		None => default
			.map(|k| Ok(ReadFilter::new(k, Box::new(writer()?))))
			.transpose(),
	}
}
