/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::collections::HashMap;

use crate::{read_filter, sink::Sink, source::Source};

pub type Tasks = HashMap<String, Task>;

#[derive(Debug)]
pub struct Task {
	pub disabled: bool,
	pub refresh: u64,
	pub tag: Option<String>,
	pub(crate) read_filter_kind: Option<read_filter::Kind>,
	pub(crate) sink: Sink,
	pub(crate) source: Source,
}

impl Task {
	#[must_use]
	pub fn new(
		disabled: bool,
		refresh: u64,
		tag: Option<String>,
		read_filter_kind: Option<read_filter::Kind>,
		sink: Sink,
		source: Source,
	) -> Self {
		// TODO: make that a Result with a custom error
		// or just remove panicing somehow else
		match (&source, &read_filter_kind) {
			(Source::Email(_), Some(_)) => {
				panic!("Email source doesn't support custom read filter types")
			}
			(Source::Email(_), None) | (_, Some(_)) => (),
			(_, None) => panic!("read_filter_type field missing"),
		}

		Self {
			disabled,
			refresh,
			tag,
			read_filter_kind,
			sink,
			source,
		}
	}
}
