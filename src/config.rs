/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

// TODO: (04.03.22) CONTINUE:
// 1. Separate each task into their own .conf files. Make Tasks into struct Tasks(HashMap<file_name, Task>)
// 2. Recursively find all .conf files in the config dir and merge them all into one
// to allow the user to structure their tasks however they want.

// TODO: add deny_unknown_fields annotations to every config struct
// TODO: mb rename .parse() into .into() or something of that sort? .into() is already used by From/Into traits though. Naming is hard, man...

pub(crate) mod auth;
mod sink;
mod source;

use serde::Deserialize;
use std::collections::HashMap;

use crate::error::Result;
use crate::task;

use self::sink::Sink;
use self::source::Source;

#[derive(Deserialize, Debug)]
#[serde(transparent, deny_unknown_fields)]
pub struct Tasks(HashMap<String, Task>);

impl Tasks {
	pub fn parse(self) -> Result<task::Tasks> {
		self.0
			.into_iter()
			.map(|(name, t)| Ok((name, t.parse()?)))
			.collect()
	}
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Task {
	pub disabled: Option<bool>,
	pub source: Source,
	pub sink: Sink,
	pub refresh: u64,
}

impl Task {
	pub(crate) fn parse(self) -> Result<task::Task> {
		Ok(task::Task {
			disabled: self.disabled,
			sink: self.sink.parse()?,
			source: self.source.parse()?,
			refresh: self.refresh,
		})
	}
}
