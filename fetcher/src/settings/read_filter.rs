/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::settings::context::StaticContext as Context;
use fetcher_config::jobs::{
	external_data::ExternalDataError,
	read_filter::{Kind as ReadFilterKind, ReadFilter as ReadFilterConf},
};
use fetcher_core::read_filter::{self as core_rf, external_save::ExternalSave, ReadFilter};

use async_trait::async_trait;
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
) -> Result<Box<dyn ReadFilter>, ExternalDataError> {
	let path = context.data_path.join(READ_DATA_DIR).join(name);

	match fs::read_to_string(&path) {
		Ok(save_file_rf_raw) if save_file_rf_raw.trim().is_empty() => {
			tracing::debug!("Read filter save file is empty");

			Ok(match expected_rf_kind {
				ReadFilterKind::NewerThanRead => Box::new(core_rf::Newer::new()),
				ReadFilterKind::NotPresentInReadList => Box::new(core_rf::NotPresent::new()),
			})
		}
		Err(e) => {
			tracing::debug!("Read filter save file doesn't exist or is inaccessible: {e}");

			Ok(match expected_rf_kind {
				ReadFilterKind::NewerThanRead => Box::new(core_rf::Newer::new()),
				ReadFilterKind::NotPresentInReadList => Box::new(core_rf::NotPresent::new()),
			})
		}
		Ok(save_file_rf_raw) => {
			let conf: ReadFilterConf =
				serde_json::from_str(&save_file_rf_raw).map_err(|e| (e, &path))?;

			// the old read filter saved on disk is of the same type as the one set in config
			if conf == expected_rf_kind {
				let rf = conf.parse(save_file(&path)?);
				Ok(rf)
			} else {
				Err(ExternalDataError::new_rf_incompat_with_path(
					expected_rf_kind,
					conf.to_kind(),
					&path,
				))
			}
		}
	}
}

// TODO: move to a new mod
#[derive(Debug)]
struct TruncatingFileWriter {
	file: fs::File,
}

// TODO: should this become async since ExternalSave is not async as well?
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

#[async_trait]
impl ExternalSave for TruncatingFileWriter {
	async fn save(&mut self, read_filter: &dyn ReadFilter) -> io::Result<()> {
		if let Some(rf_conf) =
			fetcher_config::jobs::read_filter::ReadFilter::unparse(read_filter).await
		{
			let s = serde_json::to_string(&rf_conf).unwrap();
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
