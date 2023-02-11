/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod action;
pub mod external_data;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;

pub use self::task::Task;

use fetcher_core::task::Task as CoreTask;

use std::collections::HashMap;

pub type ParsedTasks = HashMap<String, ParsedTask>;

#[derive(Debug)]
pub struct ParsedTask {
	pub inner: CoreTask,
	pub refresh: Option<u64>,
}
