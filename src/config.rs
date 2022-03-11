/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: add deny_unknown_fields annotations to every config struct
// TODO: mb rename .parse() into .into() or something of that sort? .into() is already used by From/Into traits though. Naming is hard, man... UPD: into_conf() and from_conf() are way better!

pub(crate) mod auth;
pub(crate) mod read_filter;
pub(crate) mod sink;
pub(crate) mod source;

use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::task;

use self::read_filter::Kind;
use self::sink::Sink;
use self::source::Source;

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
				if let source::html::IdQueryKind::Date = html.idq.kind {
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
