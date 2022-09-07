/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod action;
pub mod read_filter;
pub mod sink;
pub mod source;
pub mod task;

pub use self::task::Task;

use fetcher_core::read_filter::ReadFilter;
use fetcher_core::task::Task as CoreTask;

use std::collections::HashMap;

pub type ParsedTasks = HashMap<String, ParsedTask>;
pub struct ParsedTask {
	pub inner: CoreTask,
	pub refresh: u64,
}

/// A struct to pass around in the config module in order not to depend on the settings module directly
pub struct TaskSettings {
	pub twitter_auth: Option<(String, String)>,
	pub google_oauth2: Option<fetcher_core::auth::Google>,
	pub email_password: Option<String>,
	pub telegram: Option<String>,
	pub read_filter: HashMap<String, ReadFilter>,
}
