use crate::error::{Error, Result};
use std::fs;

pub struct Guid {
	pub name: &'static str,
	pub guid: String,
}

impl Guid {
	pub fn new(name: &'static str) -> Result<Self> {
		Ok(Self {
			name,
			guid: fs::read_to_string(format!("last_read_guid/{}.txt", name))
				.map_err(|e| Error::GuidGet { why: e.to_string() })?,
		})
	}

	pub fn save(self) -> Result<()> {
		let _ = fs::create_dir("last_read_guid");
		fs::write(format!("last_read_guid/{}.txt", self.name), self.guid)
			.map_err(|e| Error::GuidSave { why: e.to_string() })
	}
}
