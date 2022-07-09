use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::source;

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct File {
	path: PathBuf,
}

impl File {
	pub(crate) fn parse(self) -> source::File {
		source::File { path: self.path }
	}
}
