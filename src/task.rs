/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use teloxide::types::ChatId;

use crate::{
	error::Error,
	error::Result,
	settings,
	sink::{Sink, Telegram},
	source::Source,
};

#[derive(Deserialize)]
#[serde(transparent)]
pub struct Tasks(pub HashMap<String, Task>);

#[derive(Deserialize, Debug)]
pub struct Task {
	pub disabled: Option<bool>,
	pub sink: Sink,
	pub source: Source,
	pub refresh: u64,
}
