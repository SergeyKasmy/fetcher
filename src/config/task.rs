use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::{
	error::{Error, Result},
	task,
};

use super::{read_filter, read_filter::Kind, sink::Sink, source, source::Source};

// #[derive(Deserialize, Debug)]
// #[serde(transparent, rename = "templates")]
// pub struct Templates(pub Option<Vec<PathBuf>>);

#[derive(Deserialize, Debug)]
pub struct Templates {
	pub templates: Option<Vec<PathBuf>>,
}

#[derive(Deserialize, Debug)]
pub struct Task {
	disabled: Option<bool>,
	#[serde(rename = "read_filter_type")]
	read_filter_kind: Kind,
	refresh: u64,
	source: Source,
	sink: Sink,
}

impl Task {
	pub fn parse(self, conf_path: &Path) -> Result<task::Task> {
		if let read_filter::Kind::NewerThanRead = self.read_filter_kind {
			if let Source::Html(html) = &self.source {
				if let source::html::query::IdQueryKind::Date = html.idq.kind {
					return Err(Error::IncompatibleConfigValues(
						r#"HTML source id of type "date" isn't compatible with read_filter_type of "not_present_in_read_list""#,
						conf_path.to_owned(),
					));
				}
			}
		}
		Ok(task::Task {
			disabled: self.disabled.unwrap_or(false),
			read_filter_kind: self.read_filter_kind.parse(),
			refresh: self.refresh,
			sink: self.sink.parse()?,
			source: self.source.parse()?,
		})
	}
}
