/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;

pub type StaticContext = &'static Context;

#[derive(Debug)]
pub struct Context {
	pub data_path: PathBuf,
	pub conf_paths: Vec<PathBuf>,
	pub log_path: PathBuf,
}
