/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::fs;
use std::path::PathBuf;

use super::PREFIX;
use crate::config;
use crate::error::Error;
use crate::error::Result;
use crate::read_filter::ReadFilter;

const READ_DATA_DIR: &str = "read";

fn read_filter_path(name: &str) -> Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/read/{name}")) // FIXME
	} else {
		xdg::BaseDirectories::with_profile(PREFIX, READ_DATA_DIR)?
			.place_data_file(name)
			.map_err(|e| Error::InaccessibleData(e, format!("READ_DATA_DIR/{name}").into()))?
	})
}

/// Returns a read filter for the task name from the filesystem.
///
/// # Errors
/// * if the file is inaccessible
/// * if the file is corrupted
pub fn get(name: &str) -> Result<Option<ReadFilter>> {
	let path = read_filter_path(name)?;
	fs::read_to_string(&path)
		.ok()
		.map(|s| {
			let read_filter_conf: config::read_filter::ReadFilter = serde_json::from_str(&s)?;
			Ok(read_filter_conf.parse(name))
		})
		.transpose()
		.map_err(|e| Error::CorruptedData(e, path))
}

/// Save the provided read filter to the fs or remove it from the fs if it's empty
///
/// # Errors
/// * if the default read filter save file path is inaccessible
/// * if the write failed
/// * if the remove failed
#[allow(clippy::missing_panics_doc)]
pub fn save(read_filter: &ReadFilter) -> Result<()> {
	let path = read_filter_path(&read_filter.name)?;
	// fs::write(&path, id).map_err(|e| Error::Write(e, path))

	let read_filter_conf = config::read_filter::ReadFilter::unparse(&read_filter.inner);
	match read_filter_conf {
		Some(data) => {
			fs::write(&path, serde_json::to_string(&data).unwrap()) // unwrap NOTE: safe, serialization of such a simple struct should never fail
				.map_err(|e| Error::Write(e, path))
		}
		None => fs::remove_file(&path).map_err(|e| Error::Write(e, path)),
	}
}