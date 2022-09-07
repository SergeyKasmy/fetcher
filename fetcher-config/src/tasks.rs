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
use std::future::Future;
use std::io;
use std::pin::Pin;

pub type ParsedTasks = HashMap<String, ParsedTask>;
pub struct ParsedTask {
	pub inner: CoreTask,
	pub refresh: u64,
}

/// A struct to pass around in the config module in order not to depend on the settings module directly
/// All settings are shared except for the read_filter which is separate for each task and requires a name and a default value to get
// This one should be especially useful if we decide to move the config module out into a separate crate
pub struct TaskSettings {
	pub twitter_auth: Option<(String, String)>,
	pub google_oauth2: Option<fetcher_core::auth::Google>,
	pub email_password: Option<String>,
	pub telegram: Option<String>,
	pub read_filter: ReadFilterGetter,
}

/// A closure that takes the name of the task and its default read filter kind and returns a future
/// that returns a result if there was an error parsing current config/save file and an option if the default value is none
pub type ReadFilterGetter = Box<
	dyn Fn(
		String,
		Option<fetcher_core::read_filter::Kind>,
	) -> Pin<Box<dyn Future<Output = io::Result<Option<ReadFilter>>>>>,
>;
