use std::{fs, path::PathBuf};

use super::PREFIX;
use crate::error::{Error, Result};

const CONFIG: &str = "config.toml";

pub fn config() -> Result<String> {
	let path = if !cfg!(debug_assertions) {
		xdg::BaseDirectories::with_prefix(PREFIX)?
			.place_config_file(CONFIG)
			.map_err(Error::InaccessibleConfig)?
	} else {
		PathBuf::from(format!("debug_data/{CONFIG}"))
	};

	fs::read_to_string(&path).map_err(Error::InaccessibleConfig)
}
