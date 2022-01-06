use crate::error::{Error, Result};
use std::{fs, path::PathBuf};

pub struct Guid {
	// TODO: mb use &str here?
	pub name: String,
	pub guid: String,
	path: PathBuf,
}

impl Guid {
	pub fn new(name: &str) -> Result<Self> {
		let path = xdg::BaseDirectories::with_prefix("news_reader").unwrap().place_data_file(format!("last_read_{}.txt", name)).unwrap();	// FIXME
		Ok(Self {
			name: name.to_string(),
			// TODO: show a warning when the path doesn't exist. Mb error out when access is denied and ignore otherwise?
			guid: fs::read_to_string(&path).unwrap_or_else(|_| String::new()),
				//.map_err(|e| Error::GuidGet { why: e.to_string() })?,
			path,
		})
	}

	pub fn save(self) -> Result<()> {
		let _ = fs::create_dir("last_read_guid");
		fs::write(&self.path, self.guid)
			.map_err(|e| Error::GuidSave { why: e.to_string() })
	}
}
