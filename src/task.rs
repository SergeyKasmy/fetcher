/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::Deserialize;
use std::collections::HashMap;

use crate::{sink::Sink, source::Source};

#[derive(Deserialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct Tasks(pub HashMap<String, Task>);

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Task {
	pub disabled: Option<bool>,
	pub sink: Sink,
	pub source: Source,
	pub refresh: u64,
}
