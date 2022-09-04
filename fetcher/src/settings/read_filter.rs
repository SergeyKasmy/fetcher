/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::PREFIX;
use fetcher_config::error::ConfigError;
use fetcher_core as fcore;
use fetcher_core::read_filter::{ExternalSave, ReadFilter};

use std::path::Path;
use std::{io::Write, path::PathBuf};
use tokio::fs;

const READ_DATA_DIR: &str = "read";

/// Returns a read filter for the task name from the filesystem.
///
/// # Errors
/// * if the file is inaccessible
/// * if the file is corrupted
#[tracing::instrument(skip(currently_set_rf_kind))]
pub async fn get(
	name: &str,
	currently_set_rf_kind: Option<fcore::read_filter::Kind>,
) -> Result<Option<ReadFilter>, ConfigError> {
	match currently_set_rf_kind {
		None => Ok(None),
		Some(currently_set_rf_kind) => {
			let path = read_filter_path(name)?;

			match fs::read_to_string(&path).await {
				Ok(save_file_rf_raw) if save_file_rf_raw.trim().is_empty() => {
					tracing::debug!("Read filter save file is empty");

					Ok(Some(ReadFilter::new(
						currently_set_rf_kind,
						Box::new(save_file(&path)?),
					)))
				}
				Err(e) => {
					tracing::debug!("Read filter save file doesn't exist or is inaccessible: {e}");

					Ok(Some(ReadFilter::new(
						currently_set_rf_kind,
						Box::new(save_file(&path)?),
					)))
				}
				Ok(save_file_rf_raw) => {
					let save_file_rf = {
						let save_file_rf_conf: fetcher_config::read_filter::ReadFilter =
							serde_json::from_str(&save_file_rf_raw).map_err(|e| {
								ConfigError::CorruptedConfig(Box::new(e), path.clone())
							})?;

						save_file_rf_conf.parse(Box::new(save_file(&path)?))
					};

					// the old read filter saved on disk is of the same type as the one set in config
					if save_file_rf.to_kind() == currently_set_rf_kind {
						Ok(Some(save_file_rf))
					} else {
						Err(ConfigError::IncompatibleReadFilterTypes {
							in_config: save_file_rf.to_kind(),
							on_disk: currently_set_rf_kind,
							disk_rf_path: path,
						})
					}
				}
			}
		}
	}
}

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
		if let Some(filter_conf) = fetcher_config::read_filter::ReadFilter::unparse(read_filter) {
			let s = serde_json::to_string(&filter_conf).unwrap();
			return self.write_all(s.as_bytes());
		}

		Ok(())
	}
}

fn save_file(path: &Path) -> Result<TruncatingFileWriter, ConfigError> {
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent).map_err(|e| ConfigError::Write(e, parent.to_owned()))?;
	}

	let file = std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.open(&path)
		.map_err(|e| ConfigError::Write(e, path.to_path_buf()))?;

	Ok(TruncatingFileWriter { file })
}
