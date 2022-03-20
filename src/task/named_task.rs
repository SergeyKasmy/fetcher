/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright (C) 2022, Sergey Kasmynin (https://github.com/SergeyKasmy)
 */

use std::{
	hash::{Hash, Hasher},
	path::PathBuf,
};

use super::task::Task;

#[derive(Debug)]
pub struct NamedTask {
	pub name: String,
	pub path: PathBuf,
	pub task: Task,
}

impl Hash for NamedTask {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
	}
}

impl PartialEq for NamedTask {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl Eq for NamedTask {}
