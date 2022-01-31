use std::fs;

use crate::error::{Error, Result};

const PREFIX: &str = "fetcher";
const CONFIG: &str = "config.toml";
const LAST_READ_DATA_DIR: &str = "last-read";

pub fn get_config() -> Result<String> {
	let path = xdg::BaseDirectories::with_prefix(PREFIX)
		.map_err(|e| Error::GetData(e.to_string()))?
		.place_config_file(CONFIG)
		.map_err(|e| Error::SaveData(e.to_string()))?;

	fs::read_to_string(&path).map_err(|e| Error::GetData(e.to_string()))
}

pub fn get_last_read_id(name: &str) -> Result<String> {
	let path = xdg::BaseDirectories::with_profile(PREFIX, LAST_READ_DATA_DIR)
		.map_err(|e| Error::GetData(e.to_string()))?
		.place_data_file(name)
		.map_err(|e| Error::SaveData(e.to_string()))?;

	fs::read_to_string(path).map_err(|e| Error::GetData(e.to_string()))
}

pub fn save_last_read_id(name: &str, id: String) -> Result<()> {
	let path = xdg::BaseDirectories::with_profile(PREFIX, LAST_READ_DATA_DIR)
		.map_err(|e| Error::GetData(e.to_string()))?
		.place_data_file(name)
		.map_err(|e| Error::SaveData(e.to_string()))?;

	// fs::read_to_string(path).map_err(|e| Error::GetData(e.to_string()))
	fs::write(path, id).map_err(|e| Error::SaveData(e.to_string()))
}
