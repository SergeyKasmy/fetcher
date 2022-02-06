use std::str::FromStr;

use crate::error::{Error, Result};

#[derive(Debug)]
pub enum ViewMode {
	ReadOnly,
	MarkAsRead,
	Delete,
}

impl FromStr for ViewMode {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		Ok(match s {
			"read_only" => Self::ReadOnly,
			"mark_as_read" => Self::MarkAsRead,
			"delete" => Self::Delete,
			_ => {
				return Err(Error::ConfigInvalidFieldType {
					name: "Email".to_string(),
					field: "view_mode",
					expected_type: "string (read_only | mark_as_read | delete)",
				})
			}
		})
	}
}
