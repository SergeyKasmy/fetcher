use std::{fs, path::PathBuf};

use crate::error::{Error, Result};

const PREFIX: &str = "fetcher";
const CONFIG: &str = "config.toml";
const LAST_READ_DATA_DIR: &str = "last-read";
const TOKEN_DATA_DIR: &str = "token";

pub fn get_config() -> Result<String> {
	let path = if !cfg!(debug_assertions) {
		xdg::BaseDirectories::with_prefix(PREFIX)
			.map_err(|e| Error::GetConfig(e.to_string()))?
			.place_config_file(CONFIG)
			.map_err(|e| Error::GetConfig(e.to_string()))?
	} else {
		PathBuf::from(format!("debug_data/{CONFIG}"))
	};

	fs::read_to_string(&path).map_err(|e| Error::GetConfig(e.to_string()))
}

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

// TODO: dedup with last_read_id_path
fn token_path(name: &str) -> Result<PathBuf> {
	Ok(if cfg!(debug_assertions) {
		PathBuf::from(format!("debug_data/token/{name}"))
	} else {
		xdg::BaseDirectories::with_profile(PREFIX, TOKEN_DATA_DIR)
			.map_err(|e| Error::GetData(e.to_string()))?
			.place_data_file(name)
			.map_err(|e| Error::GetData(e.to_string()))?
	})
}

pub fn token(name: &str) -> Result<Option<String>> {
	Ok(fs::read_to_string(token_path(name)?).ok())
}

pub fn save_token(name: &str, token: &str) -> Result<()> {
	fs::write(token_path(name)?, token).map_err(|e| Error::SaveData(e.to_string()))
}
