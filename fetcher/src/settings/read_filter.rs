/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::settings::context::StaticContext as Context;
use fetcher_config::tasks::{
	external_data::ExternalDataError, read_filter::ReadFilter as ReadFilterConf,
};
use fetcher_core::read_filter::{ExternalSave, Kind as ReadFilterKind, ReadFilter};

use std::{
	fs,
	io::{self, Write},
	path::Path,
};

const READ_DATA_DIR: &str = "read";

#[tracing::instrument]
pub fn get(
	name: &str,
	expected_rf_kind: ReadFilterKind,
	context: Context,
) -> Result<ReadFilter, ExternalDataError> {
	let path = context.data_path.join(READ_DATA_DIR).join(name);

	match fs::read_to_string(&path) {
		Ok(save_file_rf_raw) if save_file_rf_raw.trim().is_empty() => {
			tracing::debug!("Read filter save file is empty");

			Ok(ReadFilter::new(
				expected_rf_kind,
				Some(Box::new(save_file(&path)?)),
			))
		}
		Err(e) => {
			tracing::debug!("Read filter save file doesn't exist or is inaccessible: {e}");

			Ok(ReadFilter::new(
				expected_rf_kind,
				Some(Box::new(save_file(&path)?)),
			))
		}
		Ok(save_file_rf_raw) => {
			let save_file_rf = {
				let conf: ReadFilterConf =
					serde_json::from_str(&save_file_rf_raw).map_err(|e| (e, &path))?;

				conf.parse(Box::new(save_file(&path)?))
			};

			// the old read filter saved on disk is of the same type as the one set in config
			if save_file_rf.to_kind() == expected_rf_kind {
				Ok(save_file_rf)
			} else {
				Err(ExternalDataError::new_rf_incompat_with_path(
					expected_rf_kind,
					save_file_rf.to_kind(),
					&path,
				))
			}
		}
	}
}

// TODO: move to a new mod
struct TruncatingFileWriter {
	file: fs::File,
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
	fn save(&mut self, read_filter: &fetcher_core::read_filter::Inner) -> io::Result<()> {
		if let Some(filter_conf) =
			fetcher_config::tasks::read_filter::ReadFilter::unparse(read_filter)
		{
			let s = serde_json::to_string(&filter_conf).unwrap();
			return self.write_all(s.as_bytes());
		}

		Ok(())
	}
}

fn save_file(path: &Path) -> io::Result<TruncatingFileWriter> {
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}

	let file = std::fs::OpenOptions::new()
		.create(true)
		.write(true)
		.open(path)?;

	Ok(TruncatingFileWriter { file })
}
