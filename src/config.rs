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
mod sink;
mod source;

use serde::Deserialize;

use crate::error::Result;
use crate::task;

use self::read_filter::ReadFilterKind;
use self::sink::Sink;
use self::source::Source;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Task {
	disabled: Option<bool>,
	#[serde(rename = "read_filter_type")]
	read_filter_kind: ReadFilterKind,
	refresh: u64,
	source: Source,
	sink: Sink,
}

impl Task {
	pub fn parse(self) -> Result<task::Task> {
		Ok(task::Task {
			disabled: self.disabled,
			read_filter_kind: self.read_filter_kind.parse(),
			refresh: self.refresh,
			sink: self.sink.parse()?,
			source: self.source.parse()?,
		})
	}
}
