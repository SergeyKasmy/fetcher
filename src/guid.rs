use crate::error::{Error, Result};
use std::fs;

pub struct Guid {
	// TODO: mb use &str here?
	pub name: String,
	pub guid: String,
}

impl Guid {
	pub fn new(name: &str) -> Result<Self> {
		Ok(Self {
			name: name.to_string(),
			// TODO: don't crash when it doesnt exist
			guid: fs::read_to_string(format!("last_read_guid/{}.txt", name))
				// TODO: show file path
				.map_err(|e| Error::GuidGet { why: e.to_string() })?,
		})
	}

	pub fn save(self) -> Result<()> {
		let _ = fs::create_dir("last_read_guid");
		fs::write(format!("last_read_guid/{}.txt", self.name), self.guid)
			.map_err(|e| Error::GuidSave { why: e.to_string() })
	}
}
