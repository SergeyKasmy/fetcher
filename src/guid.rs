use crate::error::Result;

use std::fs;

pub(crate) fn get_last_read_guid(name: &str) -> Option<String> {
	fs::read_to_string(format!("last_read_guid/{}.txt", name)).ok()
}

pub(crate) fn save_last_read_guid(name: &str, guid: String) -> Result<()> {
	let _ = fs::create_dir("last_read_guid");
	fs::write(format!("last_read_guid/{}.txt", name), guid).map_err(Into::into)
}
