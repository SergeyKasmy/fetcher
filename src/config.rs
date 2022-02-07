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
#[serde(transparent)]
pub struct Configs(pub HashMap<String, Config>);

#[derive(Deserialize, Debug)]
pub struct Config {
	pub disabled: Option<bool>,
	#[serde(flatten)]
	pub sink: Sink,
	#[serde(flatten)]
	pub source: Source,
	pub refresh: u64,
}
