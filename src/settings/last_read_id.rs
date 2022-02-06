use std::fs;
use std::path::PathBuf;

use super::PREFIX;
use crate::error::Error;
use crate::error::Result;

const LAST_READ_DATA_DIR: &str = "last-read";

fn last_read_id_path(name: &str) -> Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/last-read-id-{name}"))
	} else {
		xdg::BaseDirectories::with_profile(PREFIX, LAST_READ_DATA_DIR)
			.map_err(|e| Error::GetData(e.to_string()))?
			.place_data_file(name)
			.map_err(|e| Error::GetData(e.to_string()))?
	})
}

pub fn last_read_id(name: &str) -> Result<Option<String>> {
	Ok(fs::read_to_string(last_read_id_path(name)?).ok())
}

pub fn save_last_read_id(name: &str, id: String) -> Result<()> {
	fs::write(last_read_id_path(name)?, id).map_err(|e| Error::SaveData(e.to_string()))
}
