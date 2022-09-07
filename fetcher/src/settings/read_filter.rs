/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use super::PREFIX;
use fetcher_config::tasks::read_filter::ReadFilter as ReadFilterConf;
use fetcher_core::read_filter::{ExternalSave, ReadFilter};

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::{io::Write, path::PathBuf};

const READ_DATA_DIR: &str = "read";

/// Returns a read filter for the task name from the filesystem.
#[tracing::instrument]
pub fn get() -> io::Result<HashMap<String, ReadFilter>> {
	let mut saved_rfs = HashMap::new();

	get_all_from(&read_filter_path()?, &mut saved_rfs)?;

	Ok(saved_rfs)
}

#[tracing::instrument(skip(rfs))]
fn get_all_from(dir: &Path, rfs: &mut HashMap<String, ReadFilter>) -> io::Result<()> {
	assert!(
		dir.is_dir(),
		"Read filters can be searched for only in directories"
	);

	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let entry_path = entry.path();

		if entry_path.is_dir() {
			get_all_from(&entry_path, rfs)?;
		} else {
			let name = entry_path
				.strip_prefix(dir)
				.expect(
					"The file was found in the dir and thus should always contain it as a prefix",
				)
				.to_str()
				.ok_or_else(|| {
					io::Error::new(io::ErrorKind::Other, "File path is not valid UTF-8")
				})?
				.to_owned();

			let rf_raw = fs::read_to_string(&entry_path)?;
			if !rf_raw.is_empty() {
				let rf_conf: ReadFilterConf = serde_json::from_str(&rf_raw)?;
				let rf = rf_conf.parse(Box::new(save_file(&entry_path)?));

				tracing::trace!("Inserting ({name:?}, {rf:?}) into the hashmap");
				rfs.insert(name, rf);
			}
		}
	}

	Ok(())
}

fn read_filter_path() -> io::Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from("debug_data/read/")
	} else {
		xdg::BaseDirectories::with_profile(PREFIX, READ_DATA_DIR)?.get_data_home()
	})
}

// TODO: move to a new mod
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
	fn save(&mut self, read_filter: &fetcher_core::read_filter::ReadFilterInner) -> io::Result<()> {
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
		.open(&path)?;

	Ok(TruncatingFileWriter { file })
}
