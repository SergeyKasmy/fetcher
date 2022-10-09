/*
 * this source code form is subject to the terms of the mozilla public
 * license, v. 2.0. if a copy of the mpl was not distributed with this
 * file, you can obtain one at https://mozilla.org/mpl/2.0/.
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
	pub refresh: u64,
}
